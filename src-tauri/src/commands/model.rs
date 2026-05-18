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
    if config.model_variant != v {
        config.model_variant = v;
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
