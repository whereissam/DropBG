//! Foreground color decontamination (Phase 11.5).
//!
//! Background removal isn't only an alpha mask. A semi-transparent edge pixel
//! observes `C = α·F + (1−α)·B` — a blend of the true foreground `F` and the old
//! background `B`. If we keep `C` as the stored color and later composite onto a
//! new (e.g. white) background, the leftover `B` shows up as a colored fringe —
//! the classic green/blue halo around hair shot on a colored backdrop.
//!
//! We estimate the true foreground color by diffusing color outward from the
//! confident (opaque) core into the soft-alpha band, weighting neighbors by α²
//! so opaque foreground dominates and the background contribution is pushed out.
//! Alpha is left untouched; only edge-band color changes.

use image::{ImageBuffer, Rgba, RgbaImage};

/// Pixels at/above this alpha are trusted foreground — their color is locked and
/// used as the source the edge band is filled from.
const LOCK_ALPHA: f32 = 0.95;
/// Diffusion iterations. Each pass propagates foreground color ~1px further into
/// the band, so this bounds the fringe width that can be cleaned.
const ITERS: usize = 12;

/// Run decontamination and return `(rgb_f32, alpha_u8, w, h)` where `rgb_f32` is
/// 3 floats (0..1) per pixel and `alpha_u8` is the unchanged alpha channel.
fn decontaminate_colors(img: &RgbaImage) -> (Vec<f32>, Vec<u8>, u32, u32) {
    let (w, h) = img.dimensions();
    let n = (w * h) as usize;

    let mut color = vec![0f32; n * 3];
    let mut alpha = vec![0u8; n];
    let mut locked = vec![false; n];

    for y in 0..h {
        for x in 0..w {
            let p = img.get_pixel(x, y);
            let i = (y * w + x) as usize;
            color[i * 3] = p[0] as f32 / 255.0;
            color[i * 3 + 1] = p[1] as f32 / 255.0;
            color[i * 3 + 2] = p[2] as f32 / 255.0;
            alpha[i] = p[3];
            locked[i] = (p[3] as f32 / 255.0) >= LOCK_ALPHA;
        }
    }

    // Double-buffered alpha²-weighted diffusion into the soft band.
    let mut next = color.clone();
    for _ in 0..ITERS {
        for y in 0..h {
            for x in 0..w {
                let i = (y * w + x) as usize;
                // Locked foreground and fully-transparent pixels are left alone.
                if locked[i] || alpha[i] == 0 {
                    continue;
                }
                let mut sum = [0f32; 3];
                let mut wsum = 0f32;
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                            continue;
                        }
                        let j = ((ny as u32) * w + (nx as u32)) as usize;
                        let a = alpha[j] as f32 / 255.0;
                        // Favor opaque neighbors; give locked foreground an extra pull.
                        let weight = a * a + if locked[j] { 0.5 } else { 0.0 };
                        if weight <= 0.0 {
                            continue;
                        }
                        sum[0] += weight * color[j * 3];
                        sum[1] += weight * color[j * 3 + 1];
                        sum[2] += weight * color[j * 3 + 2];
                        wsum += weight;
                    }
                }
                if wsum > 0.0 {
                    next[i * 3] = sum[0] / wsum;
                    next[i * 3 + 1] = sum[1] / wsum;
                    next[i * 3 + 2] = sum[2] / wsum;
                }
            }
        }
        color.copy_from_slice(&next);
    }

    (color, alpha, w, h)
}

/// Decontaminated cutout as an 8-bit RGBA image (for preview / chaining).
pub fn decontaminate_rgba8(img: &RgbaImage) -> RgbaImage {
    let (color, alpha, w, h) = decontaminate_colors(img);
    let mut out = RgbaImage::new(w, h);
    for i in 0..(w * h) as usize {
        let x = (i as u32) % w;
        let y = (i as u32) / w;
        out.put_pixel(
            x,
            y,
            Rgba([
                (color[i * 3].clamp(0.0, 1.0) * 255.0).round() as u8,
                (color[i * 3 + 1].clamp(0.0, 1.0) * 255.0).round() as u8,
                (color[i * 3 + 2].clamp(0.0, 1.0) * 255.0).round() as u8,
                alpha[i],
            ]),
        );
    }
    out
}

/// Decontaminated cutout as a 16-bit RGBA image. The foreground color is encoded
/// straight from the floating-point estimate (no re-quantization banding).
///
/// When `alpha_f32` is supplied (the true full-resolution f32 alpha the model
/// produced, 0..1, length `w*h`), the alpha channel is encoded straight from it
/// for genuine 16-bit precision — this is the end-to-end 16-bit alpha path
/// (Phase 11.5 follow-up). When `None` (or the override's length doesn't match
/// the image), alpha is promoted from the 8-bit mask (0..255 → 0..65535 via
/// ×257). Color diffusion always uses the 8-bit alpha for weighting, which is
/// plenty for the edge-band color estimate.
pub fn decontaminate_rgba16_with_alpha(
    img: &RgbaImage,
    alpha_f32: Option<&[f32]>,
) -> ImageBuffer<Rgba<u16>, Vec<u16>> {
    let (color, alpha, w, h) = decontaminate_colors(img);
    let hp = alpha_f32.filter(|a| a.len() == (w * h) as usize);
    let mut out = ImageBuffer::<Rgba<u16>, Vec<u16>>::new(w, h);
    for i in 0..(w * h) as usize {
        let x = (i as u32) % w;
        let y = (i as u32) / w;
        let a16 = match hp {
            Some(a) => (a[i].clamp(0.0, 1.0) * 65535.0).round() as u16,
            None => alpha[i] as u16 * 257,
        };
        out.put_pixel(
            x,
            y,
            Rgba([
                (color[i * 3].clamp(0.0, 1.0) * 65535.0).round() as u16,
                (color[i * 3 + 1].clamp(0.0, 1.0) * 65535.0).round() as u16,
                (color[i * 3 + 2].clamp(0.0, 1.0) * 65535.0).round() as u16,
                a16,
            ]),
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opaque_pixels_keep_their_color() {
        let mut img = RgbaImage::new(2, 1);
        img.put_pixel(0, 0, Rgba([200, 100, 50, 255]));
        img.put_pixel(1, 0, Rgba([10, 20, 30, 255]));
        let out = decontaminate_rgba8(&img);
        // Fully opaque → locked → unchanged.
        assert_eq!(out.get_pixel(0, 0), &Rgba([200, 100, 50, 255]));
        assert_eq!(out.get_pixel(1, 0), &Rgba([10, 20, 30, 255]));
    }

    #[test]
    fn transparent_pixels_keep_zero_alpha() {
        let mut img = RgbaImage::new(2, 1);
        img.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        img.put_pixel(1, 0, Rgba([0, 255, 0, 0])); // transparent
        let out = decontaminate_rgba8(&img);
        assert_eq!(out.get_pixel(1, 0)[3], 0);
    }

    #[test]
    fn edge_color_pulled_toward_opaque_foreground() {
        // Opaque red | contaminated green edge | transparent.
        let mut img = RgbaImage::new(3, 1);
        img.put_pixel(0, 0, Rgba([255, 0, 0, 255])); // foreground
        img.put_pixel(1, 0, Rgba([0, 255, 0, 128])); // green-contaminated edge
        img.put_pixel(2, 0, Rgba([0, 255, 0, 0])); // background remnant
        let out = decontaminate_rgba8(&img);
        let mid = out.get_pixel(1, 0);
        // Color should shift toward the red foreground, away from green.
        assert!(mid[0] > 150, "expected red to dominate, got {:?}", mid);
        assert!(mid[1] < 150, "expected green suppressed, got {:?}", mid);
        // Alpha is untouched.
        assert_eq!(mid[3], 128);
    }

    #[test]
    fn sixteen_bit_alpha_is_scaled() {
        let mut img = RgbaImage::new(1, 1);
        img.put_pixel(0, 0, Rgba([255, 255, 255, 255]));
        let out = decontaminate_rgba16_with_alpha(&img, None);
        assert_eq!(out.get_pixel(0, 0)[3], 65535);
    }

    #[test]
    fn sixteen_bit_alpha_uses_f32_override() {
        let mut img = RgbaImage::new(1, 1);
        // 8-bit alpha 128 would promote to 128*257 = 32896.
        img.put_pixel(0, 0, Rgba([255, 255, 255, 128]));
        let out = decontaminate_rgba16_with_alpha(&img, Some(&[0.3]));
        let a = out.get_pixel(0, 0)[3];
        // True f32 precision: 0.3 → 19661, a value the 8-bit path can't produce.
        assert_eq!(a, (0.3f32 * 65535.0).round() as u16);
        assert_ne!(a, 128u16 * 257);
    }

    #[test]
    fn sixteen_bit_alpha_override_ignored_on_length_mismatch() {
        let mut img = RgbaImage::new(1, 1);
        img.put_pixel(0, 0, Rgba([0, 0, 0, 200]));
        // Wrong-length override is rejected; falls back to 8-bit promotion.
        let out = decontaminate_rgba16_with_alpha(&img, Some(&[0.1, 0.2]));
        assert_eq!(out.get_pixel(0, 0)[3], 200u16 * 257);
    }
}
