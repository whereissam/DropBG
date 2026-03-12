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
