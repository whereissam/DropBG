use super::{emit_progress, BatchProgress};
use crate::inference::face_detect::FaceDetectState;
use crate::inference::hires::{HiResCutout, HiResState};
use crate::inference::session::SessionState;
use crate::model::downloader;
use base64::Engine;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};

fn current_input_size() -> u32 {
    downloader::current_variant()
        .map(|v| v.input_size())
        .unwrap_or(1024)
}

/// If auto-routing is enabled, detect faces and return Portrait variant's input size.
/// Otherwise return the current variant's input size.
fn resolve_model_for_image(
    face_state: &FaceDetectState,
    session_state: &SessionState,
    img: &image::DynamicImage,
) -> (u32, bool) {
    let config = downloader::load_config().unwrap_or_default();
    if !config.auto_model_routing {
        return (current_input_size(), false);
    }

    // Check if Portrait model is available
    let portrait = downloader::ModelVariant::Portrait;
    let portrait_path = std::path::PathBuf::from(&config.model_dir).join(portrait.filename());
    if !portrait_path.exists() {
        return (current_input_size(), false);
    }

    // Try to detect faces
    if face_state.ensure_loaded().is_err() {
        return (current_input_size(), false);
    }

    let face_count = crate::inference::face_detect::detect_faces(face_state, img).unwrap_or(0);
    if face_count > 0 && config.model_variant != portrait {
        // Temporarily switch to portrait model
        if let Ok(()) = session_state.load_variant(&portrait) {
            return (portrait.input_size(), true);
        }
    }

    (current_input_size(), false)
}

#[tauri::command]
pub fn apple_vision_available() -> bool {
    crate::inference::apple_vision::is_available()
}

#[tauri::command]
pub async fn remove_background_apple_vision(
    app: AppHandle,
    image_path: String,
) -> Result<String, String> {
    let path = PathBuf::from(&image_path);
    if !path.exists() {
        return Err(format!("File not found: {}", image_path));
    }

    emit_progress(&app, "Running Apple Vision...", 30.0);

    let app_handle = app.clone();
    let path_str = image_path.clone();

    let result = tokio::task::spawn_blocking(move || {
        crate::inference::apple_vision::remove_background(&path_str)
    })
    .await
    .map_err(|e| e.to_string())?;

    emit_progress(&app_handle, "Done!", 100.0);
    result
}

#[tauri::command]
pub async fn remove_background(
    app: AppHandle,
    state: State<'_, SessionState>,
    face_state: State<'_, FaceDetectState>,
    hires: State<'_, HiResState>,
    image_path: String,
) -> Result<String, String> {
    let path = PathBuf::from(&image_path);
    if !path.exists() {
        return Err(format!("File not found: {}", image_path));
    }

    emit_progress(&app, "Loading model...", 5.0);
    state.ensure_loaded()?;

    let session_state = state.inner().clone();
    let face_detect_state = face_state.inner().clone();
    let hires_state = hires.inner().clone();
    let app_handle = app.clone();

    tokio::task::spawn_blocking(move || {
        emit_progress(&app_handle, "Reading image...", 15.0);
        let img = image::open(&path).map_err(|e| format!("Failed to open image: {}", e))?;
        let orig_w = img.width();
        let orig_h = img.height();

        // Auto-routing: detect faces and switch model if needed
        let (mask_size, routed) = resolve_model_for_image(&face_detect_state, &session_state, &img);
        if routed {
            emit_progress(&app_handle, "Face detected — using Portrait model...", 20.0);
        }

        // Resolve actual mask dimensions (may be non-square for dynamic models)
        let (mask_w, mask_h) = crate::inference::preprocess::resolve_mask_size(&img, mask_size);

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

        // Restore user's selected model if we auto-routed
        if routed {
            session_state.clear();
        }

        emit_progress(&app_handle, "Applying mask...", 85.0);
        let (result_img, alpha) = crate::inference::postprocess::apply_mask_rect_hp(
            &img, &mask_data, mask_w, mask_h, orig_w, orig_h,
        )?;

        emit_progress(&app_handle, "Encoding PNG...", 92.0);
        let mut buf = Vec::new();
        result_img
            .write_to(
                &mut std::io::Cursor::new(&mut buf),
                image::ImageFormat::Png,
            )
            .map_err(|e| format!("Failed to encode PNG: {}", e))?;

        let b64 = base64::engine::general_purpose::STANDARD.encode(&buf);

        // Cache the full-resolution f32 alpha keyed to this exact preview so a
        // later 16-bit export can use true alpha precision (Phase 11.5).
        hires_state.store(HiResCutout {
            preview_b64: b64.clone(),
            width: orig_w,
            height: orig_h,
            alpha,
        });

        emit_progress(&app_handle, "Done!", 100.0);
        Ok(b64)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Core background-removal pipeline operating on in-memory image bytes.
/// Shared by the batch command and the localhost HTTP API.
pub fn process_image_bytes(
    session_state: &SessionState,
    bytes: &[u8],
    mask_size: u32,
) -> Result<Vec<u8>, String> {
    let img = image::load_from_memory(bytes)
        .map_err(|e| format!("Failed to decode image: {e}"))?;
    let orig_w = img.width();
    let orig_h = img.height();

    let (mask_w, mask_h) = crate::inference::preprocess::resolve_mask_size(&img, mask_size);
    let tensor = crate::inference::preprocess::preprocess(&img, mask_size)
        .map_err(|e| e.to_string())?;

    let mask_data = {
        let mut guard = session_state
            .session
            .lock()
            .map_err(|e| format!("Session lock poisoned: {e}"))?;
        let session = guard.as_mut().ok_or("Session not initialized")?;
        crate::inference::run_inference(session, tensor)?
    };

    let result_img = crate::inference::postprocess::apply_mask_rect(
        &img, &mask_data, mask_w, mask_h, orig_w, orig_h,
    )?;

    let mut buf = Vec::new();
    result_img
        .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
        .map_err(|e| format!("Failed to encode PNG: {e}"))?;
    Ok(buf)
}

fn process_single_image(
    session_state: &SessionState,
    path: &PathBuf,
    mask_size: u32,
) -> Result<Vec<u8>, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("Failed to open image: {e}"))?;
    process_image_bytes(session_state, &bytes, mask_size)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inference::session::SessionState;

    // A 2x2 in-memory PNG so the test needs no fixture file on disk.
    fn tiny_png() -> Vec<u8> {
        let img = image::RgbaImage::from_pixel(2, 2, image::Rgba([10, 20, 30, 255]));
        let mut buf = Vec::new();
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .unwrap();
        buf
    }

    #[test]
    fn process_image_bytes_errors_without_session() {
        // No model loaded -> the session guard holds None -> "Session not initialized".
        let state = SessionState::new();
        let err = process_image_bytes(&state, &tiny_png(), 1024).unwrap_err();
        assert!(
            err.contains("Session not initialized") || err.contains("Session lock"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn process_image_bytes_rejects_garbage() {
        let state = SessionState::new();
        let err = process_image_bytes(&state, b"not an image", 1024).unwrap_err();
        assert!(err.to_lowercase().contains("decode") || err.to_lowercase().contains("image"),
            "unexpected error: {err}");
    }
}
