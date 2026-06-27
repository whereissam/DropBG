//! Inference backend selection.
//!
//! Today both backends run through ONNX Runtime; they differ only in which
//! execution provider ORT is built with:
//!   * `CoreMlEp` — ORT with the Core ML EP (Apple Neural Engine / GPU where
//!      the model's ops are supported; unsupported ops are partitioned back to
//!      CPU, and that partitioning can make it *slower* than plain CPU).
//!   * `Cpu` — ORT on CPU only.
//!
//! Because the Core ML EP is not always a win (it depends on the model's ops
//! and the machine), we don't assume — we benchmark on first use and persist
//! the fastest *correct* backend per `{model, device}`.
//!
//! A dedicated Native Core ML path (convert each model to an FP16 `.mlpackage`,
//! compile to `.mlmodelc`, run via Core ML directly) is planned but **not yet
//! implemented** — see `docs/TODO.md` Phase 11.2. When it lands it becomes a
//! third `Backend` candidate and the benchmark picks among all three.

use ndarray::Array4;
use ort::session::Session;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Instant;

use crate::model::downloader::{self, ModelVariant};

/// An inference backend the benchmark can choose between.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    /// ORT with the Core ML execution provider.
    CoreMlEp,
    /// ORT on CPU only.
    Cpu,
}

impl Default for Backend {
    fn default() -> Self {
        Backend::CoreMlEp
    }
}

impl Backend {
    pub fn key(&self) -> &'static str {
        match self {
            Backend::CoreMlEp => "coreml-ep",
            Backend::Cpu => "cpu",
        }
    }

    pub fn from_key(k: &str) -> Option<Backend> {
        match k {
            "coreml-ep" => Some(Backend::CoreMlEp),
            "cpu" => Some(Backend::Cpu),
            _ => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Backend::CoreMlEp => "Core ML (Neural Engine / GPU)",
            Backend::Cpu => "CPU",
        }
    }

    /// Backends the benchmark compares on this build. CPU is listed first so it
    /// becomes the correctness reference the Core ML output is diffed against.
    pub fn candidates() -> &'static [Backend] {
        &[Backend::Cpu, Backend::CoreMlEp]
    }
}

/// Build an ORT session for `model_path` using the given backend.
pub fn build_session(model_path: &Path, backend: Backend) -> Result<Session, String> {
    let builder =
        Session::builder().map_err(|e| format!("Failed to create session builder: {e}"))?;
    let mut builder = match backend {
        Backend::CoreMlEp => builder
            .with_execution_providers([
                ort::execution_providers::CoreMLExecutionProvider::default().build(),
            ])
            .map_err(|e| format!("Failed to set execution provider: {e}"))?,
        // No EP → ORT falls back to its CPU provider.
        Backend::Cpu => builder,
    };
    builder
        .commit_from_file(model_path)
        .map_err(|e| format!("Failed to load model: {e}"))
}

/// Best-effort machine identifier (macOS hardware model, e.g. "MacBookPro18,3").
/// Used to key benchmark results so a config synced across machines doesn't
/// apply one Mac's winning backend to a different Mac.
pub fn device_id() -> String {
    std::process::Command::new("sysctl")
        .args(["-n", "hw.model"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Config key for a benchmark result: `"{variant}:{device}"`.
pub fn bench_key(variant: &ModelVariant, device: &str) -> String {
    format!("{}:{}", variant.variant_key(), device)
}

/// Resolve which backend to use for `variant` on this machine: the persisted
/// benchmark winner if one exists, otherwise the default (Core ML EP).
pub fn resolve_backend(variant: &ModelVariant) -> Backend {
    let device = device_id();
    downloader::load_config()
        .ok()
        .and_then(|c| c.backend_benchmarks.get(&bench_key(variant, &device)).cloned())
        .and_then(|k| Backend::from_key(&k))
        .unwrap_or_default()
}

// ===== Benchmark =====

#[derive(Serialize, Clone, Debug)]
pub struct BackendTiming {
    pub backend: String, // key
    pub label: String,
    pub median_ms: f64,
    pub ok: bool,        // session built and ran
    pub diverged: bool,  // output differs from the CPU reference beyond threshold
    pub error: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct BenchmarkReport {
    pub variant: String,
    pub device: String,
    pub input_size: u32, // size actually benchmarked (dynamic models use 1024)
    pub chosen: String,  // chosen backend key
    pub timings: Vec<BackendTiming>,
}

#[derive(Serialize, Clone, Debug)]
pub struct BackendInfo {
    pub device: String,
    pub variant: String,
    pub benchmarked: bool,
    pub chosen: String,       // current backend key (default if not benchmarked)
    pub chosen_label: String,
}

/// Current backend status for the selected model (no inference run).
pub fn backend_info(variant: &ModelVariant) -> BackendInfo {
    let device = device_id();
    let persisted = downloader::load_config()
        .ok()
        .and_then(|c| c.backend_benchmarks.get(&bench_key(variant, &device)).cloned());
    let chosen = persisted
        .as_deref()
        .and_then(Backend::from_key)
        .unwrap_or_default();
    BackendInfo {
        device,
        variant: variant.variant_key().to_string(),
        benchmarked: persisted.is_some(),
        chosen: chosen.key().to_string(),
        chosen_label: chosen.label().to_string(),
    }
}

const BENCH_RUNS: usize = 3;
/// Mean-absolute-difference threshold above which a backend's output is treated
/// as divergent from the CPU reference (and therefore rejected even if faster).
const DIVERGENCE_THRESHOLD: f32 = 0.05;

fn mean_abs_diff(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return f32::INFINITY;
    }
    let sum: f32 = a.iter().zip(b).map(|(x, y)| (x - y).abs()).sum();
    sum / a.len() as f32
}

/// Build a deterministic input tensor so every backend sees identical work and
/// produces comparable output. A gentle ramp gives the model real signal rather
/// than a flat zero field.
fn synthetic_input(size: u32) -> Array4<f32> {
    let s = size as usize;
    let mut a = Array4::<f32>::zeros((1, 3, s, s));
    for (i, v) in a.iter_mut().enumerate() {
        *v = ((i % 256) as f32 / 255.0 - 0.5) * 2.0;
    }
    a
}

/// Warm up once, then time `BENCH_RUNS` runs and return (median_ms, last_output).
fn bench_one(
    model_path: &Path,
    backend: Backend,
    size: u32,
) -> Result<(f64, Vec<f32>), String> {
    let mut session = build_session(model_path, backend)?;
    // Warm-up — first run pays compilation / partitioning cost we don't want to time.
    let _ = crate::inference::run_inference(&mut session, synthetic_input(size))?;

    let mut times = Vec::with_capacity(BENCH_RUNS);
    let mut last = Vec::new();
    for _ in 0..BENCH_RUNS {
        let start = Instant::now();
        last = crate::inference::run_inference(&mut session, synthetic_input(size))?;
        times.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = times[times.len() / 2];
    Ok((median, last))
}

/// Benchmark all backend candidates for the model at `model_path` and pick the
/// fastest one whose output matches the CPU reference.
pub fn benchmark(
    variant: &ModelVariant,
    model_path: &Path,
    input_size: u32,
) -> Result<BenchmarkReport, String> {
    // Dynamic models (input_size == 0) have no fixed size; benchmark at a
    // representative 1024 so the numbers are comparable across runs.
    let size = if input_size == 0 { 1024 } else { input_size };

    let mut reference: Option<Vec<f32>> = None; // CPU output
    let mut timings: Vec<BackendTiming> = Vec::new();

    for &backend in Backend::candidates() {
        match bench_one(model_path, backend, size) {
            Ok((median_ms, output)) => {
                let diverged = match &reference {
                    Some(r) => mean_abs_diff(r, &output) > DIVERGENCE_THRESHOLD,
                    None => false, // first backend (CPU) is the reference
                };
                if reference.is_none() {
                    reference = Some(output);
                }
                timings.push(BackendTiming {
                    backend: backend.key().to_string(),
                    label: backend.label().to_string(),
                    median_ms,
                    ok: true,
                    diverged,
                    error: None,
                });
            }
            Err(e) => timings.push(BackendTiming {
                backend: backend.key().to_string(),
                label: backend.label().to_string(),
                median_ms: 0.0,
                ok: false,
                diverged: false,
                error: Some(e),
            }),
        }
    }

    // Pick the fastest backend that ran and didn't diverge; fall back to CPU.
    let chosen = timings
        .iter()
        .filter(|t| t.ok && !t.diverged)
        .min_by(|a, b| a.median_ms.partial_cmp(&b.median_ms).unwrap_or(std::cmp::Ordering::Equal))
        .map(|t| t.backend.clone())
        .unwrap_or_else(|| Backend::Cpu.key().to_string());

    Ok(BenchmarkReport {
        variant: variant.variant_key().to_string(),
        device: device_id(),
        input_size: size,
        chosen,
        timings,
    })
}
