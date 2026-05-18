use crate::model::downloader;
use base64::Engine;
use std::path::PathBuf;

#[tauri::command]
pub fn is_onboarding_done() -> bool {
    downloader::load_config().map_or(false, |c| c.onboarding_done)
}

#[tauri::command]
pub fn complete_onboarding() -> Result<(), String> {
    let mut config = downloader::load_config().map_err(|e| e.to_string())?;
    config.onboarding_done = true;
    downloader::save_config(&config).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_url_in_browser(url: String) -> Result<(), String> {
    // Only allow huggingface.co URLs
    if !url.starts_with("https://huggingface.co/") {
        return Err("Only HuggingFace URLs are allowed".to_string());
    }
    std::process::Command::new("open")
        .arg(&url)
        .spawn()
        .map_err(|e| format!("Failed to open browser: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn open_path_in_finder(path: String) -> Result<(), String> {
    let p = PathBuf::from(&path);
    // Reject paths with null bytes or parent traversal that could be suspicious
    if path.contains('\0') {
        return Err("Invalid path".to_string());
    }
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
pub async fn save_image(base64_data: String, save_path: String) -> Result<(), String> {
    // Only allow saving PNG files
    let path = PathBuf::from(&save_path);
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("png") => {}
        _ => return Err("Only .png files can be saved".to_string()),
    }

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&base64_data)
        .map_err(|e| format!("Invalid base64: {}", e))?;

    tokio::fs::write(&save_path, &bytes)
        .await
        .map_err(|e| format!("Failed to save: {}", e))?;

    Ok(())
}
