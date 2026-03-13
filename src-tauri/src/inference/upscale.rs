use crate::model::downloader;
use image::{DynamicImage, RgbaImage};
use ndarray::Array4;
use ort::session::Session;
use std::sync::{Arc, Mutex};

const TILE_SIZE: u32 = 256;
const TILE_PAD: u32 = 16;
const SCALE: u32 = 4;

#[derive(Clone)]
pub struct UpscaleSessionState {
    pub session: Arc<Mutex<Option<Session>>>,
}

unsafe impl Send for UpscaleSessionState {}
unsafe impl Sync for UpscaleSessionState {}

impl UpscaleSessionState {
    pub fn new() -> Self {
        Self {
            session: Arc::new(Mutex::new(None)),
        }
    }

    pub fn ensure_loaded(&self) -> Result<(), String> {
        let mut guard = self.session.lock().map_err(|e| format!("Upscale session lock poisoned: {e}"))?;
        if guard.is_none() {
            let model_path = downloader::upscale_model_path().map_err(|e| e.to_string())?;
            if !model_path.exists() {
                return Err(
                    "Upscale model not downloaded. Please download it in Settings.".into(),
                );
            }

            let session = Session::builder()
                .map_err(|e| format!("Failed to create session builder: {e}"))?
                .with_execution_providers([
                    ort::execution_providers::CoreMLExecutionProvider::default().build(),
                ])
                .map_err(|e| format!("Failed to set execution provider: {e}"))?
                .commit_from_file(&model_path)
                .map_err(|e| format!("Failed to load upscale model: {e}"))?;

            *guard = Some(session);
        }
        Ok(())
    }
}

/// Upscale an image 4x using Real-ESRGAN.
/// Uses tile-based processing for memory efficiency.
pub fn upscale_image(
    session_state: &UpscaleSessionState,
    img: &DynamicImage,
) -> Result<DynamicImage, String> {
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();

    // For small images, process in one pass
    if w <= TILE_SIZE && h <= TILE_SIZE {
        let result = process_tile(session_state, &rgba, 0, 0, w, h)?;
        return Ok(DynamicImage::ImageRgba8(result));
    }

    // Tile-based processing for larger images
    let out_w = w * SCALE;
    let out_h = h * SCALE;
    let mut output = RgbaImage::new(out_w, out_h);

    let mut y = 0u32;
    while y < h {
        let mut x = 0u32;
        while x < w {
            // Calculate tile bounds with padding
            let tile_x = x.saturating_sub(TILE_PAD);
            let tile_y = y.saturating_sub(TILE_PAD);
            let tile_w = (TILE_SIZE + TILE_PAD * 2).min(w - tile_x);
            let tile_h = (TILE_SIZE + TILE_PAD * 2).min(h - tile_y);

            let upscaled_tile = process_tile(session_state, &rgba, tile_x, tile_y, tile_w, tile_h)?;

            // Calculate where the non-padded region starts in the upscaled tile
            let pad_left = (x - tile_x) * SCALE;
            let pad_top = (y - tile_y) * SCALE;
            let copy_w = (TILE_SIZE.min(w - x)) * SCALE;
            let copy_h = (TILE_SIZE.min(h - y)) * SCALE;

            // Copy the non-padded region to output
            for dy in 0..copy_h {
                for dx in 0..copy_w {
                    let src_x = pad_left + dx;
                    let src_y = pad_top + dy;
                    let dst_x = x * SCALE + dx;
                    let dst_y = y * SCALE + dy;
                    if src_x < upscaled_tile.width()
                        && src_y < upscaled_tile.height()
                        && dst_x < out_w
                        && dst_y < out_h
                    {
                        output.put_pixel(dst_x, dst_y, *upscaled_tile.get_pixel(src_x, src_y));
                    }
                }
            }

            x += TILE_SIZE;
        }
        y += TILE_SIZE;
    }

    Ok(DynamicImage::ImageRgba8(output))
}

/// Process a single tile through the model.
fn process_tile(
    session_state: &UpscaleSessionState,
    img: &RgbaImage,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
) -> Result<RgbaImage, String> {
    // Extract tile
    let tile = image::imageops::crop_imm(img, x, y, w, h).to_image();

    // Convert to f32 tensor [1, 3, H, W] normalized to [0, 1]
    let mut tensor = Array4::<f32>::zeros((1, 3, h as usize, w as usize));
    for py in 0..h {
        for px in 0..w {
            let pixel = tile.get_pixel(px, py);
            tensor[[0, 0, py as usize, px as usize]] = pixel[0] as f32 / 255.0;
            tensor[[0, 1, py as usize, px as usize]] = pixel[1] as f32 / 255.0;
            tensor[[0, 2, py as usize, px as usize]] = pixel[2] as f32 / 255.0;
        }
    }

    // Run inference
    let input_value = ort::value::Value::from_array(tensor)
        .map_err(|e| format!("Failed to create input tensor: {e}"))?;

    let output_data = {
        let mut guard = session_state.session.lock().map_err(|e| format!("Upscale lock poisoned: {e}"))?;
        let session = guard.as_mut().ok_or("Upscale session not initialized")?;

        let outputs = session
            .run(ort::inputs![input_value])
            .map_err(|e| format!("Upscale inference failed: {e}"))?;

        let (_shape, data) = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|e| format!("Failed to extract output: {e}"))?;

        data.to_vec()
    };

    // Convert output tensor [1, 3, H*4, W*4] back to RGBA image
    let out_h = h * SCALE;
    let out_w = w * SCALE;
    let mut result = RgbaImage::new(out_w, out_h);

    for py in 0..out_h {
        for px in 0..out_w {
            let idx_base = py as usize * out_w as usize + px as usize;
            let ch_stride = (out_h as usize) * (out_w as usize);

            let r = (output_data[idx_base].clamp(0.0, 1.0) * 255.0) as u8;
            let g = (output_data[ch_stride + idx_base].clamp(0.0, 1.0) * 255.0) as u8;
            let b = (output_data[ch_stride * 2 + idx_base].clamp(0.0, 1.0) * 255.0) as u8;

            // Preserve alpha from the original tile (nearest-neighbor upscale)
            let orig_x = (px / SCALE).min(w - 1);
            let orig_y = (py / SCALE).min(h - 1);
            let alpha = tile.get_pixel(orig_x, orig_y)[3];

            result.put_pixel(px, py, image::Rgba([r, g, b, alpha]));
        }
    }

    Ok(result)
}
