use crate::inference::session::SessionState;
use crate::model::downloader;
use base64::Engine;
use serde::Serialize;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};

const MASK_SIZE: u32 = 1024;

#[derive(Clone, Serialize)]
struct ProcessProgress {
    step: String,
    percent: f64,
}

fn emit_progress(app: &AppHandle, step: &str, percent: f64) {
    let _ = app.emit(
        "process-progress",
        ProcessProgress {
            step: step.to_string(),
            percent,
        },
    );
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
pub fn open_path_in_finder(path: String) -> Result<(), String> {
    let p = PathBuf::from(&path);
    // Open the parent folder if it's a file, or the folder itself
    let target = if p.is_file() {
        // Use `open -R` to reveal file in Finder
        std::process::Command::new("open")
            .arg("-R")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open Finder: {e}"))?;
        return Ok(());
    } else if p.is_dir() {
        p
    } else if let Some(parent) = p.parent() {
        if parent.exists() {
            parent.to_path_buf()
        } else {
            return Err(format!("Path does not exist: {}", path));
        }
    } else {
        return Err(format!("Path does not exist: {}", path));
    };

    std::process::Command::new("open")
        .arg(target.to_string_lossy().to_string())
        .spawn()
        .map_err(|e| format!("Failed to open Finder: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn get_output_dir() -> Result<String, String> {
    downloader::output_dir()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_output_dir(new_dir: String) -> Result<(), String> {
    let mut config = downloader::load_config().map_err(|e| e.to_string())?;
    config.output_dir = new_dir;
    downloader::save_config(&config).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_model_dir(new_dir: String) -> Result<(), String> {
    let mut config = downloader::load_config().map_err(|e| e.to_string())?;
    config.model_dir = new_dir;
    downloader::save_config(&config).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_model(state: State<'_, SessionState>) -> Result<(), String> {
    let path = downloader::model_path().map_err(|e| e.to_string())?;
    if path.exists() {
        // Clear the loaded session first
        let mut guard = state.session.lock().unwrap();
        *guard = None;
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

#[tauri::command]
pub async fn remove_background(
    app: AppHandle,
    state: State<'_, SessionState>,
    image_path: String,
) -> Result<String, String> {
    let path = PathBuf::from(&image_path);
    if !path.exists() {
        return Err(format!("File not found: {}", image_path));
    }

    emit_progress(&app, "Loading model...", 5.0);
    state.ensure_loaded()?;

    let session_state = state.inner().clone();
    let app_handle = app.clone();

    tokio::task::spawn_blocking(move || {
        emit_progress(&app_handle, "Reading image...", 15.0);
        let img = image::open(&path).map_err(|e| format!("Failed to open image: {}", e))?;
        let orig_w = img.width();
        let orig_h = img.height();

        emit_progress(&app_handle, "Preprocessing...", 25.0);
        let tensor =
            crate::inference::preprocess::preprocess(&img).map_err(|e| e.to_string())?;

        emit_progress(&app_handle, "Running AI inference...", 40.0);
        let mask_data = {
            let mut guard = session_state.session.lock().unwrap();
            let session = guard.as_mut().ok_or("Session not initialized")?;
            crate::inference::run_inference(session, tensor)?
        };
        emit_progress(&app_handle, "Inference complete", 80.0);

        emit_progress(&app_handle, "Applying mask...", 85.0);
        let result_img = crate::inference::postprocess::apply_mask(
            &img, &mask_data, MASK_SIZE, orig_w, orig_h,
        )?;

        emit_progress(&app_handle, "Encoding PNG...", 92.0);
        let mut buf = Vec::new();
        result_img
            .write_to(
                &mut std::io::Cursor::new(&mut buf),
                image::ImageFormat::Png,
            )
            .map_err(|e| format!("Failed to encode PNG: {}", e))?;

        emit_progress(&app_handle, "Done!", 100.0);
        Ok(base64::engine::general_purpose::STANDARD.encode(&buf))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn save_image(base64_data: String, save_path: String) -> Result<(), String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&base64_data)
        .map_err(|e| format!("Invalid base64: {}", e))?;

    tokio::fs::write(&save_path, &bytes)
        .await
        .map_err(|e| format!("Failed to save: {}", e))?;

    Ok(())
}
