use image::{DynamicImage, GrayImage, RgbaImage};

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

/// 3x3 box blur on a grayscale image (fast approximation of Gaussian).
fn box_blur(img: &GrayImage, radius: u32) -> GrayImage {
    let (w, h) = img.dimensions();
    let mut out = GrayImage::new(w, h);
    let r = radius as i32;

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
            out.put_pixel(x, y, image::Luma([(sum / count) as u8]));
        }
    }
    out
}

/// Fill small holes in the mask: if a pixel is below threshold but surrounded
/// by high-alpha neighbors, fill it in. Uses a 5x5 neighborhood.
fn fill_small_holes(mask: &mut GrayImage, threshold: u8) {
    let (w, h) = mask.dimensions();
    let snapshot = mask.clone();

    for y in 2..h.saturating_sub(2) {
        for x in 2..w.saturating_sub(2) {
            if snapshot.get_pixel(x, y)[0] >= threshold {
                continue;
            }
            // Count how many of the 5x5 neighbors are above threshold
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
                    if snapshot.get_pixel(nx, ny)[0] >= threshold {
                        above += 1;
                    }
                }
            }
            // If >75% of neighbors are opaque, fill the hole
            if above * 4 > total * 3 {
                mask.put_pixel(x, y, image::Luma([threshold]));
            }
        }
    }
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
    // Build grayscale mask image at model resolution
    let mut mask_img = GrayImage::new(mask_w, mask_h);
    for y in 0..mask_h {
        for x in 0..mask_w {
            let idx = (y * mask_w + x) as usize;
            let val = sigmoid(mask_data[idx]);
            mask_img.put_pixel(x, y, image::Luma([(val * 255.0) as u8]));
        }
    }

    // Fill small holes at model resolution (cheaper than at full res)
    fill_small_holes(&mut mask_img, 128);

    // Resize mask to original dimensions
    let mut resized_mask = image::imageops::resize(
        &mask_img,
        orig_w,
        orig_h,
        image::imageops::FilterType::Triangle,
    );

    // Edge blur: smooth the mask edges (2px radius) for cleaner cutouts
    resized_mask = box_blur(&resized_mask, 2);

    // Apply mask as alpha channel
    let rgba = original.to_rgba8();
    let mut result = RgbaImage::new(orig_w, orig_h);

    for y in 0..orig_h {
        for x in 0..orig_w {
            let pixel = rgba.get_pixel(x, y);
            let alpha = resized_mask.get_pixel(x, y)[0];
            result.put_pixel(x, y, image::Rgba([pixel[0], pixel[1], pixel[2], alpha]));
        }
    }

    Ok(DynamicImage::ImageRgba8(result))
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
