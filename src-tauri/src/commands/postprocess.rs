use super::ProcessProgress;
use crate::inference::refine::RefineState;
use crate::inference::upscale::UpscaleSessionState;
use crate::model::downloader;
use base64::Engine;
use tauri::{AppHandle, Emitter, State};

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

// ===== Refine (ViTMatte) commands =====

#[tauri::command]
pub fn get_refine_model_info() -> Result<serde_json::Value, String> {
    let exists = crate::inference::refine::refine_model_exists();
    let path = crate::inference::refine::refine_model_path().map_err(|e| e.to_string())?;
    let size_bytes = if exists {
        std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };
    Ok(serde_json::json!({
        "name": "ViTMatte Small (quantized)",
        "exists": exists,
        "size_bytes": size_bytes,
        "approx_size": "~28 MB",
    }))
}

#[tauri::command]
pub async fn download_refine_model(app: AppHandle) -> Result<(), String> {
    if crate::inference::refine::refine_model_exists() {
        return Ok(());
    }
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        crate::inference::refine::download_refine_model(move |progress| {
            let _ = app_clone.emit("refine-download-progress", progress);
        })
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn refine_result(
    app: AppHandle,
    refine_state: State<'_, RefineState>,
    base64_data: String,
    original_path: String,
) -> Result<String, String> {
    let _ = app.emit(
        "process-progress",
        ProcessProgress {
            step: "Loading refinement model...".to_string(),
            percent: 5.0,
        },
    );
    refine_state.ensure_loaded()?;

    let state = refine_state.inner().clone();
    let app_handle = app.clone();

    tokio::task::spawn_blocking(move || {
        let _ = app_handle.emit(
            "process-progress",
            ProcessProgress {
                step: "Decoding images...".to_string(),
                percent: 10.0,
            },
        );

        // Decode the coarse result
        let coarse_bytes = base64::engine::general_purpose::STANDARD
            .decode(&base64_data)
            .map_err(|e| format!("Invalid base64: {e}"))?;
        let coarse_img = image::load_from_memory(&coarse_bytes)
            .map_err(|e| format!("Failed to decode coarse image: {e}"))?;
        let coarse_rgba = coarse_img.to_rgba8();

        // Load original image
        let original = image::open(&original_path)
            .map_err(|e| format!("Failed to open original: {e}"))?;

        let _ = app_handle.emit(
            "process-progress",
            ProcessProgress {
                step: "Refining alpha edges (ViTMatte)...".to_string(),
                percent: 30.0,
            },
        );

        let refined = crate::inference::refine::refine_mask(&state, &original, &coarse_rgba)?;

        let _ = app_handle.emit(
            "process-progress",
            ProcessProgress {
                step: "Encoding PNG...".to_string(),
                percent: 90.0,
            },
        );

        let mut buf = Vec::new();
        refined
            .write_to(
                &mut std::io::Cursor::new(&mut buf),
                image::ImageFormat::Png,
            )
            .map_err(|e| format!("Failed to encode PNG: {e}"))?;

        let _ = app_handle.emit(
            "process-progress",
            ProcessProgress {
                step: "Done!".to_string(),
                percent: 100.0,
            },
        );

        Ok(base64::engine::general_purpose::STANDARD.encode(&buf))
    })
    .await
    .map_err(|e| e.to_string())?
}

// ===== HR edge refinement (Phase 11.4) =====

/// Two-stage, tiled HR edge refinement: re-runs BiRefNet HR-matting on just the
/// uncertain soft-alpha band of an existing cutout and feather-blends it in.
/// Requires the HR-matting model to be downloaded.
#[tauri::command]
pub async fn refine_edges_hr(
    app: AppHandle,
    base64_data: String,
    original_path: String,
) -> Result<String, String> {
    let _ = app.emit(
        "process-progress",
        ProcessProgress {
            step: "Preparing HR edge refinement...".to_string(),
            percent: 5.0,
        },
    );

    let app_handle = app.clone();

    tokio::task::spawn_blocking(move || {
        let coarse_bytes = base64::engine::general_purpose::STANDARD
            .decode(&base64_data)
            .map_err(|e| format!("Invalid base64: {e}"))?;
        let coarse_img = image::load_from_memory(&coarse_bytes)
            .map_err(|e| format!("Failed to decode coarse image: {e}"))?;
        let coarse_rgba = coarse_img.to_rgba8();

        let original = image::open(&original_path)
            .map_err(|e| format!("Failed to open original: {e}"))?;

        let refined = crate::inference::hr_refine::refine_edges_hr(
            &original,
            &coarse_rgba,
            |percent, step| {
                let _ = app_handle.emit(
                    "process-progress",
                    ProcessProgress {
                        step: step.to_string(),
                        percent,
                    },
                );
            },
        )?;

        let mut buf = Vec::new();
        refined
            .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .map_err(|e| format!("Failed to encode PNG: {e}"))?;

        Ok(base64::engine::general_purpose::STANDARD.encode(&buf))
    })
    .await
    .map_err(|e| e.to_string())?
}

// ===== Foreground decontamination (Phase 11.5) =====

/// Remove background-color contamination (colored fringe) from the soft edges of
/// a cutout by estimating the true foreground color. When `sixteen_bit` is set
/// the result is encoded as a 16-bit PNG straight from the floating-point color
/// estimate (avoids re-quantization banding); otherwise an 8-bit PNG.
///
/// For the 16-bit path the alpha channel is encoded from the model's true f32
/// alpha when this exact cutout is still cached (Phase 11.5 end-to-end 16-bit);
/// if the result was edited since the cutout, it gracefully falls back to
/// promoting the 8-bit alpha.
#[tauri::command]
pub async fn decontaminate_result(
    hires: State<'_, crate::inference::hires::HiResState>,
    base64_data: String,
    sixteen_bit: bool,
) -> Result<String, String> {
    let hires_state = hires.inner().clone();
    tokio::task::spawn_blocking(move || {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(&base64_data)
            .map_err(|e| format!("Invalid base64: {e}"))?;
        let rgba = image::load_from_memory(&bytes)
            .map_err(|e| format!("Failed to decode image: {e}"))?
            .to_rgba8();
        let (w, h) = rgba.dimensions();

        let mut buf = Vec::new();
        if sixteen_bit {
            // True f32 alpha if this is the unmodified cutout; else 8-bit promotion.
            let alpha = hires_state.alpha_for(&base64_data, w, h);
            let out = crate::imaging::decontaminate::decontaminate_rgba16_with_alpha(
                &rgba,
                alpha.as_deref(),
            );
            image::DynamicImage::ImageRgba16(out)
                .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
                .map_err(|e| format!("Failed to encode 16-bit PNG: {e}"))?;
        } else {
            let out = crate::imaging::decontaminate::decontaminate_rgba8(&rgba);
            image::DynamicImage::ImageRgba8(out)
                .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
                .map_err(|e| format!("Failed to encode PNG: {e}"))?;
        }

        Ok(base64::engine::general_purpose::STANDARD.encode(&buf))
    })
    .await
    .map_err(|e| e.to_string())?
}
