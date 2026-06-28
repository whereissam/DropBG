use image::{DynamicImage, ImageBuffer, Luma, RgbaImage};

/// A continuous-tone (0..1) alpha mask at full image resolution. Keeping the
/// mask in f32 all the way through resize + blur is what makes a true 16-bit
/// alpha export possible (Phase 11.5 follow-up): the 8-bit quantization happens
/// only when composing the preview, never on the path to a 16-bit save.
type AlphaF32 = ImageBuffer<Luma<f32>, Vec<f32>>;

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

/// 3x3 box blur on an f32 grayscale image (fast approximation of Gaussian).
fn box_blur_f32(img: &AlphaF32, radius: u32) -> AlphaF32 {
    let (w, h) = img.dimensions();
    let mut out = AlphaF32::new(w, h);
    let r = radius as i32;

    for y in 0..h {
        for x in 0..w {
            let mut sum = 0f32;
            let mut count = 0f32;
            for dy in -r..=r {
                for dx in -r..=r {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx >= 0 && nx < w as i32 && ny >= 0 && ny < h as i32 {
                        sum += img.get_pixel(nx as u32, ny as u32)[0];
                        count += 1.0;
                    }
                }
            }
            out.put_pixel(x, y, Luma([sum / count]));
        }
    }
    out
}

/// Fill small holes in the mask: if a pixel is below threshold but surrounded
/// by high-alpha neighbors, fill it in. Uses a 5x5 neighborhood. Operates on a
/// flat f32 buffer (0..1) in place.
fn fill_small_holes_f32(mask: &mut [f32], w: u32, h: u32, threshold: f32) {
    let snapshot = mask.to_vec();
    let at = |x: u32, y: u32| snapshot[(y * w + x) as usize];

    for y in 2..h.saturating_sub(2) {
        for x in 2..w.saturating_sub(2) {
            if at(x, y) >= threshold {
                continue;
            }
            let mut above = 0u32;
            let mut total = 0u32;
            for dy in -2i32..=2 {
                for dx in -2i32..=2 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = (x as i32 + dx) as u32;
                    let ny = (y as i32 + dy) as u32;
                    total += 1;
                    if at(nx, ny) >= threshold {
                        above += 1;
                    }
                }
            }
            // If >75% of neighbors are opaque, fill the hole.
            if above * 4 > total * 3 {
                mask[(y * w + x) as usize] = threshold;
            }
        }
    }
}

/// Turn the raw model output into a full-resolution f32 alpha (0..1): sigmoid →
/// small-hole fill at model resolution → resize to original → 2px edge blur.
/// This is the single source of truth for alpha; both the 8-bit preview and any
/// 16-bit export are derived from it.
pub fn compute_alpha_f32(
    mask_data: &[f32],
    mask_w: u32,
    mask_h: u32,
    orig_w: u32,
    orig_h: u32,
) -> Result<Vec<f32>, String> {
    let need = (mask_w as usize) * (mask_h as usize);
    if mask_data.len() < need {
        return Err(format!(
            "mask data too short: got {}, need {} ({}x{})",
            mask_data.len(),
            need,
            mask_w,
            mask_h
        ));
    }

    let mut m = vec![0f32; need];
    for (i, slot) in m.iter_mut().enumerate() {
        *slot = sigmoid(mask_data[i]);
    }

    // Fill small holes at model resolution (cheaper than at full res).
    fill_small_holes_f32(&mut m, mask_w, mask_h, 0.5);

    let mask_img: AlphaF32 = ImageBuffer::from_raw(mask_w, mask_h, m)
        .ok_or("failed to build mask buffer")?;

    // Resize to original dimensions, then a 2px edge blur for clean edges.
    let resized = image::imageops::resize(
        &mask_img,
        orig_w,
        orig_h,
        image::imageops::FilterType::Triangle,
    );
    let blurred = box_blur_f32(&resized, 2);
    Ok(blurred.into_raw())
}

/// Compose the original RGB with an f32 alpha into an 8-bit RGBA preview.
fn compose_rgba8(original: &DynamicImage, alpha: &[f32], w: u32, h: u32) -> RgbaImage {
    let rgba = original.to_rgba8();
    let mut result = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let pixel = rgba.get_pixel(x, y);
            let a = (alpha[(y * w + x) as usize].clamp(0.0, 1.0) * 255.0).round() as u8;
            result.put_pixel(x, y, image::Rgba([pixel[0], pixel[1], pixel[2], a]));
        }
    }
    result
}

/// Apply the mask output from the model onto the original image as an alpha channel.
/// Includes edge refinement: small-hole filling + edge blur for smooth edges.
#[allow(dead_code)]
pub fn apply_mask(
    original: &DynamicImage,
    mask_data: &[f32],
    mask_size: u32,
    orig_w: u32,
    orig_h: u32,
) -> Result<DynamicImage, String> {
    apply_mask_rect(original, mask_data, mask_size, mask_size, orig_w, orig_h)
}

/// Apply mask with potentially non-square mask dimensions (for dynamic resolution).
pub fn apply_mask_rect(
    original: &DynamicImage,
    mask_data: &[f32],
    mask_w: u32,
    mask_h: u32,
    orig_w: u32,
    orig_h: u32,
) -> Result<DynamicImage, String> {
    Ok(apply_mask_rect_hp(original, mask_data, mask_w, mask_h, orig_w, orig_h)?.0)
}

/// Like [`apply_mask_rect`] but also returns the full-resolution f32 alpha so the
/// caller can cache it for a true 16-bit export (the 8-bit preview alone would
/// lose precision). The `DynamicImage` is the same 8-bit RGBA preview callers
/// already use.
pub fn apply_mask_rect_hp(
    original: &DynamicImage,
    mask_data: &[f32],
    mask_w: u32,
    mask_h: u32,
    orig_w: u32,
    orig_h: u32,
) -> Result<(DynamicImage, Vec<f32>), String> {
    let alpha = compute_alpha_f32(mask_data, mask_w, mask_h, orig_w, orig_h)?;
    let preview = compose_rgba8(original, &alpha, orig_w, orig_h);
    Ok((DynamicImage::ImageRgba8(preview), alpha))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_alpha_f32_length_and_range() {
        // 2x2 model output: strong positive logits → sigmoid ≈ 1.
        let mask = vec![8.0f32; 4];
        let alpha = compute_alpha_f32(&mask, 2, 2, 4, 4).unwrap();
        assert_eq!(alpha.len(), 16); // full-res 4x4
        for a in alpha {
            assert!(a >= 0.0 && a <= 1.0, "alpha out of range: {a}");
            assert!(a > 0.9, "expected near-opaque, got {a}");
        }
    }

    #[test]
    fn compute_alpha_f32_rejects_short_input() {
        // Need 4 values for 2x2, give 3.
        let mask = vec![0.0f32; 3];
        assert!(compute_alpha_f32(&mask, 2, 2, 2, 2).is_err());
    }

    #[test]
    fn compute_alpha_f32_negative_logits_are_transparent() {
        let mask = vec![-8.0f32; 4];
        let alpha = compute_alpha_f32(&mask, 2, 2, 2, 2).unwrap();
        for a in alpha {
            assert!(a < 0.1, "expected near-transparent, got {a}");
        }
    }
}

/// Auto-crop: trim fully transparent edges, with optional padding.
pub fn autocrop(img: &DynamicImage, padding: u32) -> DynamicImage {
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();

    let (x, y, cw, ch) = crate::imaging::autocrop::autocrop(&rgba);

    // Apply padding
    let x = x.saturating_sub(padding);
    let y = y.saturating_sub(padding);
    let cw = (cw + padding * 2).min(w - x);
    let ch = (ch + padding * 2).min(h - y);

    DynamicImage::ImageRgba8(image::imageops::crop_imm(&rgba, x, y, cw, ch).to_image())
}
