use image::{DynamicImage, GrayImage, RgbaImage};
use ndarray::Array4;
use ort::session::Session;
use std::sync::{Arc, Mutex};

const REFINE_MODEL_URL: &str =
    "https://huggingface.co/Xenova/vitmatte-small-composition-1k/resolve/main/onnx/model_quantized.onnx";
const REFINE_MODEL_FILENAME: &str = "vitmatte_small_q.onnx";

// ImageNet normalization constants (used by ViTMatte)
const MEAN: [f32; 3] = [0.485, 0.456, 0.406];
const STD: [f32; 3] = [0.229, 0.224, 0.225];

#[derive(Clone)]
pub struct RefineState {
    pub session: Arc<Mutex<Option<Session>>>,
}

unsafe impl Send for RefineState {}
unsafe impl Sync for RefineState {}

impl RefineState {
    pub fn new() -> Self {
        Self {
            session: Arc::new(Mutex::new(None)),
        }
    }

    pub fn ensure_loaded(&self) -> Result<(), String> {
        let mut guard = self
            .session
            .lock()
            .map_err(|e| format!("Refine lock poisoned: {e}"))?;
        if guard.is_none() {
            let path = refine_model_path().map_err(|e| e.to_string())?;
            if !path.exists() {
                return Err("Refinement model not downloaded. Download it in Settings.".into());
            }

            let session = Session::builder()
                .map_err(|e| format!("Failed to create session builder: {e}"))?
                .commit_from_file(&path)
                .map_err(|e| format!("Failed to load refine model: {e}"))?;

            *guard = Some(session);
        }
        Ok(())
    }
}

pub fn refine_model_path() -> anyhow::Result<std::path::PathBuf> {
    let config = crate::model::downloader::load_config()?;
    Ok(std::path::PathBuf::from(&config.model_dir).join(REFINE_MODEL_FILENAME))
}

pub fn refine_model_exists() -> bool {
    refine_model_path().map_or(false, |p| p.exists())
}

pub fn download_refine_model<F>(on_progress: F) -> anyhow::Result<()>
where
    F: Fn(f64) + Send + 'static,
{
    let dest = refine_model_path()?;
    if dest.exists() {
        return Ok(());
    }
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let client = reqwest::blocking::Client::new();
    let mut resp = client.get(REFINE_MODEL_URL).send()?;
    if !resp.status().is_success() {
        anyhow::bail!("Download failed with status: {}", resp.status());
    }

    let total_size = resp.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let tmp_path = dest.with_extension("onnx.tmp");
    let mut file = std::fs::File::create(&tmp_path)?;

    let mut buffer = [0u8; 8192];
    loop {
        let bytes_read = std::io::Read::read(&mut resp, &mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        std::io::Write::write_all(&mut file, &buffer[..bytes_read])?;
        downloaded += bytes_read as u64;
        if total_size > 0 {
            on_progress((downloaded as f64 / total_size as f64) * 100.0);
        }
    }

    std::io::Write::flush(&mut file)?;
    drop(file);
    std::fs::rename(&tmp_path, &dest)?;
    Ok(())
}

/// Generate a trimap from a coarse mask.
/// - White (255) = definite foreground (mask > high_threshold)
/// - Black (0) = definite background (mask < low_threshold)
/// - Gray (128) = uncertain region (the edge band)
fn generate_trimap(mask: &GrayImage, edge_width: u32) -> GrayImage {
    let (w, h) = mask.dimensions();
    let mut trimap = GrayImage::new(w, h);

    // First pass: classify pixels
    for y in 0..h {
        for x in 0..w {
            let val = mask.get_pixel(x, y)[0];
            if val > 220 {
                trimap.put_pixel(x, y, image::Luma([255])); // definite fg
            } else if val < 30 {
                trimap.put_pixel(x, y, image::Luma([0])); // definite bg
            } else {
                trimap.put_pixel(x, y, image::Luma([128])); // uncertain
            }
        }
    }

    // Dilate the uncertain region by edge_width
    let snapshot = trimap.clone();
    let r = edge_width as i32;
    for y in 0..h {
        for x in 0..w {
            let val = snapshot.get_pixel(x, y)[0];
            if val == 128 {
                continue; // already uncertain
            }
            // Check if near an edge
            let mut near_edge = false;
            'outer: for dy in -r..=r {
                for dx in -r..=r {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx >= 0 && nx < w as i32 && ny >= 0 && ny < h as i32 {
                        let neighbor = snapshot.get_pixel(nx as u32, ny as u32)[0];
                        if (val == 255 && neighbor < 30) || (val == 0 && neighbor > 220) {
                            near_edge = true;
                            break 'outer;
                        }
                    }
                }
            }
            if near_edge {
                trimap.put_pixel(x, y, image::Luma([128]));
            }
        }
    }

    trimap
}

/// Run ViTMatte to refine a coarse mask into a true alpha matte.
///
/// Input: original image + coarse mask (from BiRefNet or similar)
/// Output: refined RGBA image with soft alpha edges
pub fn refine_mask(
    state: &RefineState,
    original: &DynamicImage,
    coarse_rgba: &RgbaImage,
) -> Result<DynamicImage, String> {
    let (w, h) = coarse_rgba.dimensions();

    // Extract coarse alpha as grayscale mask
    let mut coarse_mask = GrayImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            coarse_mask.put_pixel(x, y, image::Luma([coarse_rgba.get_pixel(x, y)[3]]));
        }
    }

    // Generate trimap from coarse mask
    let trimap = generate_trimap(&coarse_mask, 15);

    // Check if there's actually an uncertain region worth refining
    let uncertain_count = (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .filter(|&(x, y)| trimap.get_pixel(x, y)[0] == 128)
        .count();

    if uncertain_count == 0 {
        // No uncertain region — return original as-is
        return Ok(DynamicImage::ImageRgba8(coarse_rgba.clone()));
    }

    // ViTMatte input: [1, 4, H, W] — 3 RGB channels + 1 trimap channel
    // RGB normalized with ImageNet mean/std, trimap normalized to [0, 1]
    let rgb = original.resize_exact(w, h, image::imageops::FilterType::Triangle).to_rgb8();

    let mut tensor = Array4::<f32>::zeros((1, 4, h as usize, w as usize));
    for y in 0..h as usize {
        for x in 0..w as usize {
            let pixel = rgb.get_pixel(x as u32, y as u32);
            // RGB channels: ImageNet normalized
            tensor[[0, 0, y, x]] = (pixel[0] as f32 / 255.0 - MEAN[0]) / STD[0];
            tensor[[0, 1, y, x]] = (pixel[1] as f32 / 255.0 - MEAN[1]) / STD[1];
            tensor[[0, 2, y, x]] = (pixel[2] as f32 / 255.0 - MEAN[2]) / STD[2];
            // Trimap channel: normalized to [0, 1]
            tensor[[0, 3, y, x]] = trimap.get_pixel(x as u32, y as u32)[0] as f32 / 255.0;
        }
    }

    // Run ViTMatte inference
    let input_value = ort::value::Value::from_array(tensor)
        .map_err(|e| format!("Failed to create refine input tensor: {e}"))?;

    let alpha_data = {
        let mut guard = state
            .session
            .lock()
            .map_err(|e| format!("Refine lock poisoned: {e}"))?;
        let session = guard.as_mut().ok_or("Refine session not loaded")?;

        let outputs = session
            .run(ort::inputs![input_value])
            .map_err(|e| format!("Refine inference failed: {e}"))?;

        let (_shape, data) = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|e| format!("Failed to extract refine output: {e}"))?;

        data.to_vec()
    };

    // Build refined RGBA image
    let orig_rgba = original
        .resize_exact(w, h, image::imageops::FilterType::Triangle)
        .to_rgba8();
    let mut result = RgbaImage::new(w, h);

    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            let alpha = if idx < alpha_data.len() {
                (alpha_data[idx].clamp(0.0, 1.0) * 255.0) as u8
            } else {
                coarse_rgba.get_pixel(x, y)[3]
            };
            let pixel = orig_rgba.get_pixel(x, y);
            result.put_pixel(x, y, image::Rgba([pixel[0], pixel[1], pixel[2], alpha]));
        }
    }

    Ok(DynamicImage::ImageRgba8(result))
}
