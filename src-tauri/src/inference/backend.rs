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
//! implemented** — see `docs/TODO.md` Phase 11.2b. When it lands it becomes a
//! third `Backend` candidate and the benchmark picks among all three.
//!
//! ## Precision policy (11.2b)
//! The curated ONNX models ship as **FP16** weights, so inference already runs
//! at half precision on every backend here. Apple Silicon prefers FP16 on the
//! Neural Engine / GPU; that's the default we report. Everything *around*
//! inference — mask resize, normalization, alpha compositing — is kept in FP32
//! (see `postprocess::compute_alpha_f32`) to avoid alpha banding. We don't ship
//! an FP32 ONNX variant to A/B against, so the benchmark records the precision
//! it ran (`fp16`) rather than comparing precisions.

use ndarray::Array4;
use ort::session::Session;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::model::downloader::{self, ModelVariant};

/// The precision string recorded in benchmark results.
///
/// The curated ONNX models ship as FP16 weights, so inference runs at FP16 on
/// every backend here regardless of platform — Apple Silicon executes FP16
/// natively on the Neural Engine / GPU, and on Intel ORT loads the same FP16
/// weights. There is no FP32 ONNX variant to A/B against, so this is `"fp16"`
/// today; it's a function so a future native FP32 path can report `"fp32"`
/// without touching call sites.
pub fn precision_label() -> &'static str {
    "fp16"
}

// ===== Peak-memory sampling =====
//
// Measuring per-backend peak memory precisely would need mach `task_info`
// plumbing; we get an honest, dependency-free approximation by polling the
// process RSS (`ps -o rss=`) on a background thread while a backend runs and
// keeping the max. It's process-wide (not backend-isolated), but since the
// benchmark loads one session at a time the delta over the idle baseline tracks
// that backend's footprint closely enough to compare candidates.

/// Current resident set size of this process in MB, via `ps`. Returns 0.0 if the
/// platform/command is unavailable (the field is then simply uninformative).
fn current_rss_mb() -> f64 {
    let pid = std::process::id();
    std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &pid.to_string()])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<f64>().ok())
        .map(|kb| kb / 1024.0)
        .unwrap_or(0.0)
}

/// Polls process RSS on a background thread and records the peak until stopped.
struct MemSampler {
    stop: Arc<AtomicBool>,
    peak_kb: Arc<AtomicU64>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl MemSampler {
    fn start() -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let peak_kb = Arc::new(AtomicU64::new(0));
        let (s, p) = (stop.clone(), peak_kb.clone());
        let handle = std::thread::spawn(move || {
            while !s.load(Ordering::Relaxed) {
                let kb = (current_rss_mb() * 1024.0) as u64;
                p.fetch_max(kb, Ordering::Relaxed);
                std::thread::sleep(Duration::from_millis(8));
            }
        });
        MemSampler { stop, peak_kb, handle: Some(handle) }
    }

    /// Stop sampling and return the peak RSS observed, in MB.
    fn stop(mut self) -> f64 {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
        self.peak_kb.load(Ordering::Relaxed) as f64 / 1024.0
    }
}

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
    pub peak_memory_mb: f64, // peak process RSS observed while this backend ran (0 if unavailable)
    pub ok: bool,        // session built and ran
    pub diverged: bool,  // output differs from the CPU reference beyond threshold
    pub error: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct BenchmarkReport {
    pub variant: String,
    pub device: String,
    pub input_size: u32, // size actually benchmarked (dynamic models use 1024)
    pub precision: String, // inference precision the models ran at (fp16)
    pub chosen: String,  // chosen backend key
    pub timings: Vec<BackendTiming>,
    /// Human-readable caveat, e.g. Core ML ran but was slower than CPU (op
    /// partitioning penalty) so CPU was chosen. `None` when the result is clean.
    pub note: Option<String>,
}

/// Rich, persisted benchmark record for the winning backend. Stored alongside
/// the simple winner-key map so the UI can show latency / memory / precision
/// without re-running, and so 11.2b's `{ median_ms, peak_memory_mb, precision }`
/// record shape is captured. Optional on `AppConfig` (serde default) so configs
/// written before this field load unchanged.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BackendRecord {
    pub backend: String,
    pub median_ms: f64,
    pub peak_memory_mb: f64,
    pub precision: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct BackendInfo {
    pub device: String,
    pub variant: String,
    pub benchmarked: bool,
    pub chosen: String,       // current backend key (default if not benchmarked)
    pub chosen_label: String,
    pub precision: String,
    /// True when the selected model is a dynamic-shape / matting model that has
    /// not been benchmarked on this machine yet. These are the models whose ops
    /// can silently partition the Core ML EP back to CPU, so the UI prompts the
    /// user to benchmark before trusting the default backend.
    pub needs_benchmark: bool,
    /// The persisted rich record for the chosen backend, if benchmarked.
    pub record: Option<BackendRecord>,
}

/// Current backend status for the selected model (no inference run).
pub fn backend_info(variant: &ModelVariant) -> BackendInfo {
    let device = device_id();
    let config = downloader::load_config().ok();
    let key = bench_key(variant, &device);
    let persisted = config
        .as_ref()
        .and_then(|c| c.backend_benchmarks.get(&key).cloned());
    let record = config
        .as_ref()
        .and_then(|c| c.backend_records.get(&key).cloned());
    let chosen = persisted
        .as_deref()
        .and_then(Backend::from_key)
        .unwrap_or_default();
    let benchmarked = persisted.is_some();
    // Dynamic / matting models are the ones whose ops can partition the Core ML
    // EP to CPU; flag them for benchmarking until proven on this machine.
    let needs_benchmark =
        !benchmarked && (variant.is_dynamic() || variant.is_matting_model());
    BackendInfo {
        device,
        variant: variant.variant_key().to_string(),
        benchmarked,
        chosen: chosen.key().to_string(),
        chosen_label: chosen.label().to_string(),
        precision: precision_label().to_string(),
        needs_benchmark,
        record,
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

/// Warm up once, then time `BENCH_RUNS` runs and return
/// `(median_ms, peak_memory_mb, last_output)`. Peak memory is sampled across the
/// whole build+run window so it includes the loaded session's footprint.
fn bench_one(
    model_path: &Path,
    backend: Backend,
    size: u32,
) -> Result<(f64, f64, Vec<f32>), String> {
    let sampler = MemSampler::start();
    let result = (|| {
        let mut session = build_session(model_path, backend)?;
        // Warm-up — first run pays compilation / partitioning cost we don't time.
        let _ = crate::inference::run_inference(&mut session, synthetic_input(size))?;

        let mut times = Vec::with_capacity(BENCH_RUNS);
        let mut last = Vec::new();
        for _ in 0..BENCH_RUNS {
            let start = Instant::now();
            last = crate::inference::run_inference(&mut session, synthetic_input(size))?;
            times.push(start.elapsed().as_secs_f64() * 1000.0);
        }
        times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        Ok::<_, String>((times[times.len() / 2], last))
    })();
    let peak_mb = sampler.stop();
    let (median, last) = result?;
    Ok((median, peak_mb, last))
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
            Ok((median_ms, peak_memory_mb, output)) => {
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
                    peak_memory_mb,
                    ok: true,
                    diverged,
                    error: None,
                });
            }
            Err(e) => timings.push(BackendTiming {
                backend: backend.key().to_string(),
                label: backend.label().to_string(),
                median_ms: 0.0,
                peak_memory_mb: 0.0,
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

    // Explain a non-obvious outcome: the Core ML EP ran fine but lost to CPU,
    // which is exactly the op-partitioning penalty 11.2 warns about. Surfacing
    // it is the "safe promotion" signal for dynamic/matting models.
    let cpu = timings.iter().find(|t| t.backend == Backend::Cpu.key());
    let coreml = timings.iter().find(|t| t.backend == Backend::CoreMlEp.key());
    let note = match (cpu, coreml) {
        (Some(c), Some(m)) if m.ok && c.ok && !m.diverged && chosen == c.backend && m.median_ms > c.median_ms => {
            Some(format!(
                "Core ML ran but was slower than CPU ({:.0} ms vs {:.0} ms) — likely op partitioning back to CPU. Using CPU.",
                m.median_ms, c.median_ms
            ))
        }
        (_, Some(m)) if m.ok && m.diverged => {
            Some("Core ML output diverged from the CPU reference and was rejected. Using CPU.".to_string())
        }
        _ => None,
    };

    Ok(BenchmarkReport {
        variant: variant.variant_key().to_string(),
        device: device_id(),
        input_size: size,
        precision: precision_label().to_string(),
        chosen,
        timings,
        note,
    })
}
