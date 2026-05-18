use super::{emit_progress, BatchProgress};
use crate::inference::cloud_usage::CloudUsageState;
use crate::model::downloader;
use base64::Engine;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub fn get_cloud_config() -> Result<serde_json::Value, String> {
    let config = downloader::load_config().map_err(|e| e.to_string())?;
    let providers: Vec<serde_json::Value> = downloader::CloudProvider::all()
        .iter()
        .map(|p| {
            serde_json::json!({
                "key": p.variant_key(),
                "name": p.name(),
                "description": p.description(),
            })
        })
        .collect();

    let fal_ai_endpoints: Vec<serde_json::Value> = downloader::FalAIEndpoint::all()
        .iter()
        .map(|e| {
            serde_json::json!({
                "key": e.variant_key(),
                "name": e.name(),
                "description": e.description(),
            })
        })
        .collect();

    Ok(serde_json::json!({
        "enabled": config.cloud_enabled,
        "provider": config.cloud_provider.variant_key(),
        "provider_name": config.cloud_provider.name(),
        "has_api_key": config.has_cloud_api_key(),
        "providers": providers,
        "fal_ai_endpoint": config.fal_ai_endpoint.variant_key(),
        "fal_ai_endpoint_name": config.fal_ai_endpoint.name(),
        "fal_ai_endpoints": fal_ai_endpoints,
    }))
}

#[tauri::command]
pub fn set_fal_ai_endpoint(endpoint: String) -> Result<(), String> {
    let e = downloader::FalAIEndpoint::from_key(&endpoint)
        .ok_or_else(|| format!("Unknown fal.ai endpoint: {endpoint}"))?;
    let mut config = downloader::load_config().map_err(|err| err.to_string())?;
    config.fal_ai_endpoint = e;
    downloader::save_config(&config).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn set_cloud_enabled(enabled: bool) -> Result<(), String> {
    let mut config = downloader::load_config().map_err(|e| e.to_string())?;
    config.cloud_enabled = enabled;
    downloader::save_config(&config).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_cloud_provider(provider: String) -> Result<(), String> {
    let p = downloader::CloudProvider::from_key(&provider)
        .ok_or_else(|| format!("Unknown provider: {provider}"))?;
    let mut config = downloader::load_config().map_err(|e| e.to_string())?;
    config.cloud_provider = p;
    downloader::save_config(&config).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_cloud_api_key(api_key: String) -> Result<(), String> {
    let mut config = downloader::load_config().map_err(|e| e.to_string())?;
    let provider_key = config.cloud_provider.variant_key().to_string();
    config.cloud_api_keys.insert(provider_key, api_key);
    downloader::save_config(&config).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_cloud_usage(
    usage_state: State<'_, CloudUsageState>,
) -> Result<serde_json::Value, String> {
    let summary = usage_state.summary();
    serde_json::to_value(&summary).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reset_cloud_usage(usage_state: State<'_, CloudUsageState>) {
    usage_state.reset();
}

#[tauri::command]
pub async fn remove_background_cloud(
    app: AppHandle,
    usage_state: State<'_, CloudUsageState>,
    image_path: String,
) -> Result<String, String> {
    let path = PathBuf::from(&image_path);
    if !path.exists() {
        return Err(format!("File not found: {}", image_path));
    }

    let app_handle = app.clone();
    let usage = usage_state.inner().clone();

    tokio::task::spawn_blocking(move || {
        emit_progress(&app_handle, "Reading image...", 10.0);
        let image_bytes = std::fs::read(&path)
            .map_err(|e| format!("Failed to read image: {e}"))?;

        let config = downloader::load_config().map_err(|e| e.to_string())?;
        let provider = config.cloud_provider.clone();

        emit_progress(&app_handle, "Uploading to cloud API...", 25.0);
        let result_bytes = crate::inference::cloud::remove_background_cloud(&image_bytes)?;

        // Record successful usage
        usage.record(&provider);

        emit_progress(&app_handle, "Processing result...", 85.0);

        // Ensure result is valid PNG — re-encode through image crate
        let img = image::load_from_memory(&result_bytes)
            .map_err(|e| format!("Failed to decode cloud result: {e}"))?;
        let rgba = img.to_rgba8();

        emit_progress(&app_handle, "Encoding PNG...", 92.0);
        let mut buf = Vec::new();
        image::DynamicImage::ImageRgba8(rgba)
            .write_to(
                &mut std::io::Cursor::new(&mut buf),
                image::ImageFormat::Png,
            )
            .map_err(|e| format!("Failed to encode PNG: {e}"))?;

        emit_progress(&app_handle, "Done!", 100.0);
        Ok(base64::engine::general_purpose::STANDARD.encode(&buf))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn remove_background_batch_cloud(
    app: AppHandle,
    usage_state: State<'_, CloudUsageState>,
    image_paths: Vec<String>,
    output_dir: String,
) -> Result<Vec<String>, String> {
    if image_paths.is_empty() {
        return Ok(vec![]);
    }

    let app_handle = app.clone();
    let usage = usage_state.inner().clone();
    let total = image_paths.len();

    tokio::task::spawn_blocking(move || {
        let config = downloader::load_config().map_err(|e| e.to_string())?;
        let provider = config.cloud_provider.clone();
        let mut output_paths = Vec::new();
        let out_dir = PathBuf::from(&output_dir);
        std::fs::create_dir_all(&out_dir)
            .map_err(|e| format!("Failed to create output dir: {e}"))?;

        for (i, image_path) in image_paths.iter().enumerate() {
            let path = PathBuf::from(image_path);
            let filename = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| format!("image_{i}"));
            let display_name = path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| image_path.clone());

            let _ = app_handle.emit(
                "batch-progress",
                BatchProgress {
                    index: i,
                    total,
                    filename: display_name.clone(),
                    status: "processing".to_string(),
                    error: None,
                    output_path: None,
                },
            );

            let result = (|| -> Result<Vec<u8>, String> {
                let image_bytes = std::fs::read(&path)
                    .map_err(|e| format!("Failed to read image: {e}"))?;
                let result_bytes = crate::inference::cloud::remove_background_cloud(&image_bytes)?;
                usage.record(&provider);
                // Re-encode through image crate to ensure valid PNG
                let img = image::load_from_memory(&result_bytes)
                    .map_err(|e| format!("Failed to decode cloud result: {e}"))?;
                let mut buf = Vec::new();
                image::DynamicImage::ImageRgba8(img.to_rgba8())
                    .write_to(
                        &mut std::io::Cursor::new(&mut buf),
                        image::ImageFormat::Png,
                    )
                    .map_err(|e| format!("Failed to encode PNG: {e}"))?;
                Ok(buf)
            })();

            match result {
                Ok(png_bytes) => {
                    let out_path = out_dir.join(format!("{}_nobg.png", filename));
                    match std::fs::write(&out_path, &png_bytes) {
                        Ok(_) => {
                            let out_str = out_path.to_string_lossy().to_string();
                            output_paths.push(out_str.clone());
                            let _ = app_handle.emit(
                                "batch-progress",
                                BatchProgress {
                                    index: i,
                                    total,
                                    filename: display_name,
                                    status: "done".to_string(),
                                    error: None,
                                    output_path: Some(out_str),
                                },
                            );
                        }
                        Err(e) => {
                            let _ = app_handle.emit(
                                "batch-progress",
                                BatchProgress {
                                    index: i,
                                    total,
                                    filename: display_name,
                                    status: "error".to_string(),
                                    error: Some(format!("Save failed: {e}")),
                                    output_path: None,
                                },
                            );
                        }
                    }
                }
                Err(e) => {
                    let _ = app_handle.emit(
                        "batch-progress",
                        BatchProgress {
                            index: i,
                            total,
                            filename: display_name,
                            status: "error".to_string(),
                            error: Some(e),
                            output_path: None,
                        },
                    );
                }
            }
        }

        Ok(output_paths)
    })
    .await
    .map_err(|e| e.to_string())?
}
