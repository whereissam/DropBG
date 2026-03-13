use crate::inference::session::SessionState;
use crate::inference::upscale::UpscaleSessionState;
use crate::model::downloader;
use base64::Engine;
use serde::Serialize;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};

fn current_input_size() -> u32 {
    downloader::current_variant()
        .map(|v| v.input_size())
        .unwrap_or(1024)
}

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

    let mask_size = current_input_size();

    tokio::task::spawn_blocking(move || {
        emit_progress(&app_handle, "Reading image...", 15.0);
        let img = image::open(&path).map_err(|e| format!("Failed to open image: {}", e))?;
        let orig_w = img.width();
        let orig_h = img.height();

        emit_progress(&app_handle, "Preprocessing...", 25.0);
        let tensor =
            crate::inference::preprocess::preprocess(&img, mask_size).map_err(|e| e.to_string())?;

        emit_progress(&app_handle, "Running AI inference...", 40.0);
        let mask_data = {
            let mut guard = session_state.session.lock().map_err(|e| format!("Session lock poisoned: {e}"))?;
            let session = guard.as_mut().ok_or("Session not initialized")?;
            crate::inference::run_inference(session, tensor)?
        };
        emit_progress(&app_handle, "Inference complete", 80.0);

        emit_progress(&app_handle, "Applying mask...", 85.0);
        let result_img = crate::inference::postprocess::apply_mask(
            &img, &mask_data, mask_size, orig_w, orig_h,
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

#[derive(Clone, Serialize)]
struct BatchProgress {
    index: usize,
    total: usize,
    filename: String,
    status: String, // "processing" | "done" | "error"
    error: Option<String>,
    output_path: Option<String>,
}

fn process_single_image(
    session_state: &SessionState,
    path: &PathBuf,
    mask_size: u32,
) -> Result<Vec<u8>, String> {
    let img = image::open(path).map_err(|e| format!("Failed to open image: {}", e))?;
    let orig_w = img.width();
    let orig_h = img.height();

    let tensor = crate::inference::preprocess::preprocess(&img, mask_size).map_err(|e| e.to_string())?;

    let mask_data = {
        let mut guard = session_state.session.lock().map_err(|e| format!("Session lock poisoned: {e}"))?;
        let session = guard.as_mut().ok_or("Session not initialized")?;
        crate::inference::run_inference(session, tensor)?
    };

    let result_img =
        crate::inference::postprocess::apply_mask(&img, &mask_data, mask_size, orig_w, orig_h)?;

    let mut buf = Vec::new();
    result_img
        .write_to(
            &mut std::io::Cursor::new(&mut buf),
            image::ImageFormat::Png,
        )
        .map_err(|e| format!("Failed to encode PNG: {}", e))?;

    Ok(buf)
}

#[tauri::command]
pub async fn remove_background_batch(
    app: AppHandle,
    state: State<'_, SessionState>,
    image_paths: Vec<String>,
    output_dir: String,
) -> Result<Vec<String>, String> {
    if image_paths.is_empty() {
        return Ok(vec![]);
    }

    state.ensure_loaded()?;

    let session_state = state.inner().clone();
    let app_handle = app.clone();
    let total = image_paths.len();
    let mask_size = current_input_size();

    tokio::task::spawn_blocking(move || {
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

            match process_single_image(&session_state, &path, mask_size) {
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

#[tauri::command]
pub fn replace_background_color(
    base64_data: String,
    r: u8, g: u8, b: u8,
) -> Result<String, String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&base64_data)
        .map_err(|e| format!("Invalid base64: {e}"))?;
    let img = image::load_from_memory(&bytes)
        .map_err(|e| format!("Failed to decode image: {e}"))?;
    let rgba = img.to_rgba8();

    let result = crate::imaging::background::replace_with_color(&rgba, r, g, b);
    encode_rgba_to_base64(&result)
}

#[tauri::command]
pub fn replace_background_gradient(
    base64_data: String,
    r1: u8, g1: u8, b1: u8,
    r2: u8, g2: u8, b2: u8,
) -> Result<String, String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&base64_data)
        .map_err(|e| format!("Invalid base64: {e}"))?;
    let img = image::load_from_memory(&bytes)
        .map_err(|e| format!("Failed to decode image: {e}"))?;
    let rgba = img.to_rgba8();

    let result = crate::imaging::background::replace_with_gradient(&rgba, r1, g1, b1, r2, g2, b2);
    encode_rgba_to_base64(&result)
}

#[tauri::command]
pub fn replace_background_image(
    base64_data: String,
    bg_image_path: String,
) -> Result<String, String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&base64_data)
        .map_err(|e| format!("Invalid base64: {e}"))?;
    let fg = image::load_from_memory(&bytes)
        .map_err(|e| format!("Failed to decode foreground: {e}"))?;

    let bg = image::open(&bg_image_path)
        .map_err(|e| format!("Failed to open background image: {e}"))?;

    let result = crate::imaging::background::replace_with_image(&fg.to_rgba8(), &bg);
    encode_rgba_to_base64(&result)
}

fn encode_rgba_to_base64(img: &image::RgbaImage) -> Result<String, String> {
    let dyn_img = image::DynamicImage::ImageRgba8(img.clone());
    let mut buf = Vec::new();
    dyn_img
        .write_to(
            &mut std::io::Cursor::new(&mut buf),
            image::ImageFormat::Png,
        )
        .map_err(|e| format!("Failed to encode PNG: {e}"))?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&buf))
}

#[tauri::command]
pub fn auto_crop(base64_data: String, padding: Option<u32>) -> Result<String, String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&base64_data)
        .map_err(|e| format!("Invalid base64: {e}"))?;

    let img = image::load_from_memory(&bytes).map_err(|e| format!("Failed to decode image: {e}"))?;
    let cropped = crate::inference::postprocess::autocrop(&img, padding.unwrap_or(4));

    let mut buf = Vec::new();
    cropped
        .write_to(
            &mut std::io::Cursor::new(&mut buf),
            image::ImageFormat::Png,
        )
        .map_err(|e| format!("Failed to encode PNG: {e}"))?;

    Ok(base64::engine::general_purpose::STANDARD.encode(&buf))
}

// ===== Upscale commands =====

#[tauri::command]
pub fn get_upscale_model_info() -> Result<downloader::UpscaleModelInfo, String> {
    downloader::upscale_model_info().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn download_upscale_model(app: AppHandle) -> Result<(), String> {
    if downloader::upscale_model_exists() {
        return Ok(());
    }

    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        downloader::download_upscale_model(move |progress| {
            let _ = app_clone.emit("upscale-download-progress", progress);
        })
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn upscale_image(
    app: AppHandle,
    state: State<'_, UpscaleSessionState>,
    base64_data: String,
    scale: Option<u32>, // 2 or 4, default 4
) -> Result<String, String> {
    let _ = app.emit("process-progress", ProcessProgress {
        step: "Loading upscale model...".to_string(),
        percent: 5.0,
    });
    state.ensure_loaded()?;

    let upscale_state = state.inner().clone();
    let app_handle = app.clone();
    let target_scale = scale.unwrap_or(4);

    tokio::task::spawn_blocking(move || {
        let _ = app_handle.emit("process-progress", ProcessProgress {
            step: "Decoding image...".to_string(),
            percent: 10.0,
        });

        let bytes = base64::engine::general_purpose::STANDARD
            .decode(&base64_data)
            .map_err(|e| format!("Invalid base64: {e}"))?;
        let img = image::load_from_memory(&bytes)
            .map_err(|e| format!("Failed to decode image: {e}"))?;

        let _ = app_handle.emit("process-progress", ProcessProgress {
            step: "Upscaling (AI super-resolution)...".to_string(),
            percent: 20.0,
        });

        let upscaled = crate::inference::upscale::upscale_image(&upscale_state, &img)?;

        // If 2x requested, downscale from 4x to 2x
        let final_img = if target_scale == 2 {
            let _ = app_handle.emit("process-progress", ProcessProgress {
                step: "Resizing to 2x...".to_string(),
                percent: 85.0,
            });
            let (w, h) = (img.width() * 2, img.height() * 2);
            let resized = image::imageops::resize(
                &upscaled.to_rgba8(),
                w, h,
                image::imageops::FilterType::Lanczos3,
            );
            image::DynamicImage::ImageRgba8(resized)
        } else {
            upscaled
        };

        let _ = app_handle.emit("process-progress", ProcessProgress {
            step: "Encoding PNG...".to_string(),
            percent: 92.0,
        });

        let mut buf = Vec::new();
        final_img
            .write_to(
                &mut std::io::Cursor::new(&mut buf),
                image::ImageFormat::Png,
            )
            .map_err(|e| format!("Failed to encode PNG: {e}"))?;

        let _ = app_handle.emit("process-progress", ProcessProgress {
            step: "Done!".to_string(),
            percent: 100.0,
        });

        Ok(base64::engine::general_purpose::STANDARD.encode(&buf))
    })
    .await
    .map_err(|e| e.to_string())?
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
