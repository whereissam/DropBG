use image::{DynamicImage, RgbaImage};

/// Composite foreground (with alpha) over a solid color background.
pub fn replace_with_color(img: &RgbaImage, r: u8, g: u8, b: u8) -> RgbaImage {
    let (w, h) = img.dimensions();
    let mut result = RgbaImage::new(w, h);

    for y in 0..h {
        for x in 0..w {
            let pixel = img.get_pixel(x, y);
            let alpha = pixel[3] as f32 / 255.0;
            let inv = 1.0 - alpha;

            let out_r = (pixel[0] as f32 * alpha + r as f32 * inv) as u8;
            let out_g = (pixel[1] as f32 * alpha + g as f32 * inv) as u8;
            let out_b = (pixel[2] as f32 * alpha + b as f32 * inv) as u8;

            result.put_pixel(x, y, image::Rgba([out_r, out_g, out_b, 255]));
        }
    }

    result
}

/// Composite foreground over a linear gradient background (top-to-bottom).
pub fn replace_with_gradient(
    img: &RgbaImage,
    r1: u8, g1: u8, b1: u8,
    r2: u8, g2: u8, b2: u8,
) -> RgbaImage {
    let (w, h) = img.dimensions();
    let mut result = RgbaImage::new(w, h);
    let h_f = h as f32;

    for y in 0..h {
        let t = y as f32 / h_f;
        let bg_r = (r1 as f32 * (1.0 - t) + r2 as f32 * t) as u8;
        let bg_g = (g1 as f32 * (1.0 - t) + g2 as f32 * t) as u8;
        let bg_b = (b1 as f32 * (1.0 - t) + b2 as f32 * t) as u8;

        for x in 0..w {
            let pixel = img.get_pixel(x, y);
            let alpha = pixel[3] as f32 / 255.0;
            let inv = 1.0 - alpha;

            let out_r = (pixel[0] as f32 * alpha + bg_r as f32 * inv) as u8;
            let out_g = (pixel[1] as f32 * alpha + bg_g as f32 * inv) as u8;
            let out_b = (pixel[2] as f32 * alpha + bg_b as f32 * inv) as u8;

            result.put_pixel(x, y, image::Rgba([out_r, out_g, out_b, 255]));
        }
    }

    result
}

/// Composite foreground over a custom image background.
/// The background image is resized to match the foreground dimensions.
pub fn replace_with_image(fg: &RgbaImage, bg: &DynamicImage) -> RgbaImage {
    let (w, h) = fg.dimensions();
    let bg_resized = image::imageops::resize(
        &bg.to_rgba8(),
        w,
        h,
        image::imageops::FilterType::Triangle,
    );
    let mut result = RgbaImage::new(w, h);

    for y in 0..h {
        for x in 0..w {
            let fg_px = fg.get_pixel(x, y);
            let bg_px = bg_resized.get_pixel(x, y);
            let alpha = fg_px[3] as f32 / 255.0;
            let inv = 1.0 - alpha;

            let out_r = (fg_px[0] as f32 * alpha + bg_px[0] as f32 * inv) as u8;
            let out_g = (fg_px[1] as f32 * alpha + bg_px[1] as f32 * inv) as u8;
            let out_b = (fg_px[2] as f32 * alpha + bg_px[2] as f32 * inv) as u8;

            result.put_pixel(x, y, image::Rgba([out_r, out_g, out_b, 255]));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    fn fg_pixel(r: u8, g: u8, b: u8, a: u8) -> RgbaImage {
        let mut img = RgbaImage::new(1, 1);
        img.put_pixel(0, 0, Rgba([r, g, b, a]));
        img
    }

    #[test]
    fn fully_opaque_fg_preserves_color_alpha_255() {
        let fg = fg_pixel(100, 150, 200, 255);
        let out = replace_with_color(&fg, 0, 0, 0);
        let p = out.get_pixel(0, 0);
        assert_eq!(p[0], 100);
        assert_eq!(p[1], 150);
        assert_eq!(p[2], 200);
        assert_eq!(p[3], 255);
    }

    #[test]
    fn fully_transparent_fg_shows_only_background() {
        let fg = fg_pixel(255, 255, 255, 0);
        let out = replace_with_color(&fg, 42, 84, 168);
        let p = out.get_pixel(0, 0);
        assert_eq!(p[0], 42);
        assert_eq!(p[1], 84);
        assert_eq!(p[2], 168);
        assert_eq!(p[3], 255);
    }

    #[test]
    fn half_alpha_blends_evenly() {
        let fg = fg_pixel(255, 0, 0, 128);
        let out = replace_with_color(&fg, 0, 0, 255);
        let p = out.get_pixel(0, 0);
        // 255 * (128/255) + 0  ≈ 128
        // 0   * (128/255) + 255*(1 - 128/255) ≈ 127
        assert!((p[0] as i32 - 128).abs() <= 1, "red ≈ 128, got {}", p[0]);
        assert!(p[1] <= 1, "green ≈ 0, got {}", p[1]);
        assert!((p[2] as i32 - 127).abs() <= 2, "blue ≈ 127, got {}", p[2]);
        assert_eq!(p[3], 255);
    }

    #[test]
    fn gradient_interpolates_top_to_bottom() {
        // 1×10 transparent fg → result reveals the gradient
        let mut fg = RgbaImage::new(1, 10);
        for px in fg.pixels_mut() {
            *px = Rgba([0, 0, 0, 0]);
        }
        let out = replace_with_gradient(&fg, 0, 0, 0, 255, 255, 255);
        let top = out.get_pixel(0, 0);
        let bot = out.get_pixel(0, 9);
        assert_eq!(top[0], 0);
        // bottom should be near full white (t = 9/10 = 0.9 → 229)
        assert!(bot[0] >= 220, "bottom red should be near 255 — got {}", bot[0]);
        // monotonic non-decreasing
        let mut prev = 0u8;
        for y in 0..10 {
            let v = out.get_pixel(0, y)[0];
            assert!(v >= prev, "gradient not monotonic at y={y}");
            prev = v;
        }
    }

    #[test]
    fn output_dimensions_match_foreground() {
        let mut fg = RgbaImage::new(13, 7);
        for px in fg.pixels_mut() {
            *px = Rgba([255, 255, 255, 255]);
        }
        let out = replace_with_color(&fg, 0, 0, 0);
        assert_eq!(out.dimensions(), (13, 7));

        let out2 = replace_with_gradient(&fg, 0, 0, 0, 255, 0, 0);
        assert_eq!(out2.dimensions(), (13, 7));
    }
}
