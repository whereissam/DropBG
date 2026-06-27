//! Edge-only HR refinement (Phase 11.4) — two-stage, tiled.
//!
//! The default pipeline produces a coarse mask at the model's input size
//! (1024 / dynamic). Running a heavy high-resolution matting model over a whole
//! 6000×4000 image is slow and memory-hungry; downscaling everything throws away
//! the very detail we want. Instead:
//!
//!   Stage 1: coarse mask (already produced by the selected model).
//!   Stage 2: find the *uncertain* band (soft-alpha edges: hair, fur, glass),
//!            cut it into overlapping high-res tiles, run **BiRefNet HR-matting**
//!            on those tiles only, and feather-blend the refined alpha back in.
//!
//! Confident interior / background pixels keep the coarse alpha, so the heavy
//! model only pays for the hard edges and peak memory stays a few tiles, not a
//! full high-res forward pass.

use image::{DynamicImage, GrayImage, RgbaImage};

use crate::inference::backend;
use crate::model::downloader::{self, ModelVariant};

/// Tile size cut from the original image. Each tile is resized up to the
/// HR-matting input (2048²) before inference, so a smallish tile still gets a
/// high-res forward pass over its edges.
const TILE: u32 = 512;
/// Overlap between adjacent tiles; the blend feathers across this band so tile
/// seams don't show.
const OVERLAP: u32 = 128;

/// Alpha band treated as "uncertain" (soft edge worth refining). Pixels fully
/// opaque or fully transparent are left to the coarse mask.
const BAND_LOW: u8 = 16;
const BAND_HIGH: u8 = 240;
/// Dilate the uncertain band by this many pixels so a tile fully covers each edge.
const BAND_DILATE: u32 = 12;

/// Refine the soft edges of a coarse result with tiled HR-matting.
///
/// `original` is the full-resolution source image; `coarse_rgba` is the current
/// cutout (its alpha channel is the coarse mask). Returns a new RGBA image with
/// refined edge alpha. If HR-matting isn't downloaded, returns an error.
pub fn refine_edges_hr(
    original: &DynamicImage,
    coarse_rgba: &RgbaImage,
    mut on_progress: impl FnMut(f64, &str),
) -> Result<DynamicImage, String> {
    let (w, h) = coarse_rgba.dimensions();
    let orig_rgba = if original.width() == w && original.height() == h {
        original.to_rgba8()
    } else {
        original
            .resize_exact(w, h, image::imageops::FilterType::Triangle)
            .to_rgba8()
    };

    // Coarse alpha as grayscale.
    let mut coarse_alpha = GrayImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            coarse_alpha.put_pixel(x, y, image::Luma([coarse_rgba.get_pixel(x, y)[3]]));
        }
    }

    // --- Uncertainty map: soft band, dilated, then feathered to a 0..1 weight ---
    let uncertain = build_uncertainty_mask(&coarse_alpha);
    let active_tiles = tiles_covering(&uncertain, w, h);
    if active_tiles.is_empty() {
        // Nothing soft to refine — hand back the coarse result unchanged.
        return Ok(DynamicImage::ImageRgba8(coarse_rgba.clone()));
    }

    on_progress(15.0, "Loading HR-matting model...");
    let mut session = load_hr_matting_session()?;

    // Accumulators for feathered overlap-blend of refined alpha.
    let n = (w * h) as usize;
    let mut acc = vec![0f32; n]; // sum(weight * refined_alpha)
    let mut wsum = vec![0f32; n]; // sum(weight)

    let total = active_tiles.len();
    for (i, &(tx, ty, tw, th)) in active_tiles.iter().enumerate() {
        on_progress(
            20.0 + 60.0 * (i as f64 / total as f64),
            &format!("Refining edge tile {}/{}", i + 1, total),
        );

        let tile = image::imageops::crop_imm(&orig_rgba, tx, ty, tw, th).to_image();
        let tile_img = DynamicImage::ImageRgba8(tile);
        let refined = run_hr_tile(&mut session, &tile_img, tw, th)?;

        // Feather window for this tile (linear ramp over the overlap band).
        for ly in 0..th {
            for lx in 0..tw {
                let weight = feather(lx, tw) * feather(ly, th);
                let gx = tx + lx;
                let gy = ty + ly;
                let idx = (gy * w + gx) as usize;
                let a = refined.get_pixel(lx, ly)[0] as f32 / 255.0;
                acc[idx] += weight * a;
                wsum[idx] += weight;
            }
        }
    }

    on_progress(85.0, "Blending refined edges...");
    let mut result = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            let coarse = coarse_alpha.get_pixel(x, y)[0] as f32 / 255.0;
            let u = uncertain.get_pixel(x, y)[0] as f32 / 255.0; // soft 0..1 weight

            let final_alpha = if wsum[idx] > 0.0 && u > 0.0 {
                let refined = acc[idx] / wsum[idx];
                // Blend coarse → refined only in the uncertain band, feathered by u.
                coarse * (1.0 - u) + refined * u
            } else {
                coarse
            };

            let p = orig_rgba.get_pixel(x, y);
            result.put_pixel(
                x,
                y,
                image::Rgba([p[0], p[1], p[2], (final_alpha.clamp(0.0, 1.0) * 255.0) as u8]),
            );
        }
    }

    on_progress(100.0, "Done!");
    Ok(DynamicImage::ImageRgba8(result))
}

/// Build a soft 0..1 (stored 0..255) uncertainty weight: 1 in the soft-alpha
/// band (dilated), fading to 0 in confident regions so the blend has no seam.
fn build_uncertainty_mask(coarse_alpha: &GrayImage) -> GrayImage {
    let (w, h) = coarse_alpha.dimensions();
    let mut band = GrayImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let a = coarse_alpha.get_pixel(x, y)[0];
            let v = if a > BAND_LOW && a < BAND_HIGH { 255 } else { 0 };
            band.put_pixel(x, y, image::Luma([v]));
        }
    }
    dilate(&mut band, BAND_DILATE);
    // Feather the binary band into a smooth weight.
    box_blur(&band, BAND_DILATE / 2 + 1)
}

/// Grid of overlapping tiles that contain at least one uncertain pixel.
/// Returns `(x, y, w, h)` clamped to image bounds.
fn tiles_covering(uncertain: &GrayImage, w: u32, h: u32) -> Vec<(u32, u32, u32, u32)> {
    let stride = TILE - OVERLAP;
    let mut out = Vec::new();
    let mut ty = 0u32;
    while ty < h {
        let th = TILE.min(h - ty);
        let mut tx = 0u32;
        while tx < w {
            let tw = TILE.min(w - tx);
            if tile_has_uncertain(uncertain, tx, ty, tw, th) {
                out.push((tx, ty, tw, th));
            }
            if tx + tw >= w {
                break;
            }
            tx += stride;
        }
        if ty + th >= h {
            break;
        }
        ty += stride;
    }
    out
}

fn tile_has_uncertain(uncertain: &GrayImage, tx: u32, ty: u32, tw: u32, th: u32) -> bool {
    // Sample on a coarse stride — we only need to know if any edge passes through.
    let step = 4;
    let mut y = ty;
    while y < ty + th {
        let mut x = tx;
        while x < tx + tw {
            if uncertain.get_pixel(x, y)[0] > 8 {
                return true;
            }
            x += step;
        }
        y += step;
    }
    false
}

/// Run HR-matting on a single tile and return its refined alpha at tile size.
fn run_hr_tile(
    session: &mut ort::session::Session,
    tile_img: &DynamicImage,
    tw: u32,
    th: u32,
) -> Result<GrayImage, String> {
    let input_size = ModelVariant::HRMatting.input_size(); // 2048
    let tensor = crate::inference::preprocess::preprocess(tile_img, input_size)
        .map_err(|e| e.to_string())?;
    let mask = crate::inference::run_inference(session, tensor)?;

    // Model output is `input_size² ` logits → sigmoid → grayscale → resize to tile.
    let mut full = GrayImage::new(input_size, input_size);
    for y in 0..input_size {
        for x in 0..input_size {
            let idx = (y * input_size + x) as usize;
            let v = if idx < mask.len() { sigmoid(mask[idx]) } else { 0.0 };
            full.put_pixel(x, y, image::Luma([(v * 255.0) as u8]));
        }
    }
    Ok(image::imageops::resize(
        &full,
        tw,
        th,
        image::imageops::FilterType::Triangle,
    ))
}

fn load_hr_matting_session() -> Result<ort::session::Session, String> {
    let variant = ModelVariant::HRMatting;
    let config = downloader::load_config().map_err(|e| e.to_string())?;
    let path = std::path::PathBuf::from(&config.model_dir).join(variant.filename());
    if !path.exists() {
        return Err(
            "BiRefNet HR-matting isn't downloaded. Add it in Settings → Advanced (export script) to use HR edge refinement."
                .into(),
        );
    }
    backend::build_session(&path, backend::resolve_backend(&variant))
}

// ===== small helpers =====

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

/// Linear feather weight in [0,1]: ramps up over the first `OVERLAP` pixels and
/// back down over the last `OVERLAP`, flat 1.0 in the middle. Keeps a small
/// floor so a 1-tile pixel still gets nonzero weight.
fn feather(pos: u32, len: u32) -> f32 {
    if len <= 1 {
        return 1.0;
    }
    let o = OVERLAP.min(len / 2).max(1) as f32;
    let p = pos as f32;
    let last = (len - 1) as f32;
    let rise = (p + 1.0) / o;
    let fall = (last - p + 1.0) / o;
    rise.min(fall).min(1.0).max(0.02)
}

/// Grow nonzero regions by `radius` (chebyshev) — a cheap binary dilation.
fn dilate(img: &mut GrayImage, radius: u32) {
    if radius == 0 {
        return;
    }
    let (w, h) = img.dimensions();
    let src = img.clone();
    let r = radius as i32;
    for y in 0..h {
        for x in 0..w {
            if src.get_pixel(x, y)[0] > 0 {
                continue;
            }
            let mut hit = false;
            'o: for dy in -r..=r {
                for dx in -r..=r {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx >= 0 && nx < w as i32 && ny >= 0 && ny < h as i32 && src.get_pixel(nx as u32, ny as u32)[0] > 0 {
                        hit = true;
                        break 'o;
                    }
                }
            }
            if hit {
                img.put_pixel(x, y, image::Luma([255]));
            }
        }
    }
}

/// Separable-ish box blur (used to feather the uncertainty band).
fn box_blur(img: &GrayImage, radius: u32) -> GrayImage {
    if radius == 0 {
        return img.clone();
    }
    let (w, h) = img.dimensions();
    let r = radius as i32;
    let mut out = GrayImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let mut sum = 0u32;
            let mut count = 0u32;
            for dy in -r..=r {
                for dx in -r..=r {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx >= 0 && nx < w as i32 && ny >= 0 && ny < h as i32 {
                        sum += img.get_pixel(nx as u32, ny as u32)[0] as u32;
                        count += 1;
                    }
                }
            }
            out.put_pixel(x, y, image::Luma([(sum / count.max(1)) as u8]));
        }
    }
    out
}
