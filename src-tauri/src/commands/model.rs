use crate::inference::face_detect::FaceDetectState;
use crate::inference::session::SessionState;
use crate::model::downloader;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub fn get_auto_routing() -> bool {
    downloader::load_config().map_or(false, |c| c.auto_model_routing)
}

#[tauri::command]
pub fn set_auto_routing(enabled: bool, face_state: State<'_, FaceDetectState>) -> Result<(), String> {
    let mut config = downloader::load_config().map_err(|e| e.to_string())?;
    config.auto_model_routing = enabled;
    downloader::save_config(&config).map_err(|e| e.to_string())?;

    if enabled {
        // Download face detection model if not present (~233 KB)
        if !crate::inference::face_detect::face_model_exists() {
            crate::inference::face_detect::download_face_model().map_err(|e| e.to_string())?;
        }
        face_state.ensure_loaded()?;
    }
    Ok(())
}

#[tauri::command]
pub fn check_model_ready() -> bool {
    downloader::model_path().map_or(false, |p| p.exists())
}

#[tauri::command]
pub fn get_model_info() -> Result<downloader::ModelInfo, String> {
    downloader::get_model_info().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_model_dir(new_dir: String) -> Result<(), String> {
    let mut config = downloader::load_config().map_err(|e| e.to_string())?;
    config.model_dir = new_dir;
    downloader::save_config(&config).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_model_variant(variant: String, state: State<'_, SessionState>) -> Result<(), String> {
    let v = downloader::ModelVariant::from_key(&variant)
        .ok_or_else(|| format!("Unknown variant: {variant}"))?;
    let mut config = downloader::load_config().map_err(|e| e.to_string())?;
    // Picking a raw model from the Advanced list drops out of any preset mode.
    let mode_changed = config.processing_mode != downloader::ProcessingMode::Advanced;
    if config.model_variant != v || mode_changed {
        config.model_variant = v;
        config.processing_mode = downloader::ProcessingMode::Advanced;
        downloader::save_config(&config).map_err(|e| e.to_string())?;
        // Clear loaded session so next inference reloads the new model
        state.clear();
    }
    Ok(())
}

#[tauri::command]
pub fn delete_model(state: State<'_, SessionState>) -> Result<(), String> {
    let path = downloader::model_path().map_err(|e| e.to_string())?;
    if path.exists() {
        state.clear();
        std::fs::remove_file(&path).map_err(|e| format!("Failed to delete model: {e}"))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn download_model(app: AppHandle) -> Result<(), String> {
    let path = downloader::model_path().map_err(|e| e.to_string())?;
    if path.exists() {
        return Ok(());
    }

    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        downloader::download_model(&path, move |progress| {
            let _ = app_clone.emit("download-progress", progress);
        })
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

// ===== Inference backend benchmark (Phase 11.2) =====

/// Current backend status for the selected model — no inference is run.
#[tauri::command]
pub fn get_backend_info() -> Result<crate::inference::backend::BackendInfo, String> {
    let config = downloader::load_config().map_err(|e| e.to_string())?;
    Ok(crate::inference::backend::backend_info(&config.model_variant))
}

/// Benchmark the available inference backends (Core ML EP vs CPU) for the
/// currently selected model, persist the fastest *correct* one for this
/// machine, and clear the session so the next inference uses it.
#[tauri::command]
pub async fn benchmark_inference_backends(
    state: State<'_, SessionState>,
) -> Result<crate::inference::backend::BenchmarkReport, String> {
    let report = tokio::task::spawn_blocking(|| -> Result<_, String> {
        let config = downloader::load_config().map_err(|e| e.to_string())?;
        let variant = config.model_variant.clone();
        let path = downloader::model_path().map_err(|e| e.to_string())?;
        if !path.exists() {
            return Err(format!(
                "{} is not downloaded — download it before benchmarking.",
                variant.name()
            ));
        }

        let report =
            crate::inference::backend::benchmark(&variant, &path, variant.input_size())?;

        // Persist the winner for this {variant, device} — both the simple key
        // (consulted when building a session) and the rich record (latency /
        // memory / precision, for the Settings UI).
        let mut config = downloader::load_config().map_err(|e| e.to_string())?;
        let key = crate::inference::backend::bench_key(&variant, &report.device);
        config.backend_benchmarks.insert(key.clone(), report.chosen.clone());
        if let Some(w) = report.timings.iter().find(|t| t.backend == report.chosen) {
            config.backend_records.insert(
                key,
                crate::inference::backend::BackendRecord {
                    backend: w.backend.clone(),
                    median_ms: w.median_ms,
                    peak_memory_mb: w.peak_memory_mb,
                    precision: report.precision.clone(),
                },
            );
        }
        downloader::save_config(&config).map_err(|e| e.to_string())?;

        Ok(report)
    })
    .await
    .map_err(|e| e.to_string())??;

    // Force the next session build to honor the freshly chosen backend.
    state.clear();
    Ok(report)
}

// ===== Processing modes (Phase 11.3) =====

#[derive(serde::Serialize)]
pub struct ProcessingModeOption {
    pub key: String,
    pub label: String,
    pub description: String,
    pub variant: Option<String>, // variant_key, or null for Apple Vision / Advanced
    pub uses_apple_vision: bool,
}

#[derive(serde::Serialize)]
pub struct ProcessingModeInfo {
    pub current: String,
    pub modes: Vec<ProcessingModeOption>,
}

/// The four user-facing modes plus the currently selected one.
#[tauri::command]
pub fn get_processing_mode() -> Result<ProcessingModeInfo, String> {
    let config = downloader::load_config().map_err(|e| e.to_string())?;
    let modes = downloader::ProcessingMode::user_modes()
        .iter()
        .map(|m| ProcessingModeOption {
            key: m.key().to_string(),
            label: m.label().to_string(),
            description: m.description().to_string(),
            variant: m.variant().map(|v| v.variant_key().to_string()),
            uses_apple_vision: m.uses_apple_vision(),
        })
        .collect();
    Ok(ProcessingModeInfo {
        current: config.processing_mode.key().to_string(),
        modes,
    })
}

/// Select a processing mode. Non-Fast modes also set the underlying model
/// variant; Fast routes through Apple Vision (handled in the frontend).
#[tauri::command]
pub fn set_processing_mode(mode: String, state: State<'_, SessionState>) -> Result<(), String> {
    let m = downloader::ProcessingMode::from_key(&mode)
        .ok_or_else(|| format!("Unknown mode: {mode}"))?;
    let mut config = downloader::load_config().map_err(|e| e.to_string())?;
    config.processing_mode = m.clone();
    if let Some(v) = m.variant() {
        config.model_variant = v;
    }
    downloader::save_config(&config).map_err(|e| e.to_string())?;
    state.clear();
    Ok(())
}
