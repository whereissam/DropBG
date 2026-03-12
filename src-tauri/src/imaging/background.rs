use image::RgbaImage;

#[allow(dead_code)]
pub fn replace_with_color(img: &RgbaImage, r: u8, g: u8, b: u8) -> RgbaImage {
    let (w, h) = img.dimensions();
    let mut result = RgbaImage::new(w, h);

    for y in 0..h {
        for x in 0..w {
            let pixel = img.get_pixel(x, y);
            let alpha = pixel[3] as f32 / 255.0;
            let inv_alpha = 1.0 - alpha;

            let out_r = (pixel[0] as f32 * alpha + r as f32 * inv_alpha) as u8;
            let out_g = (pixel[1] as f32 * alpha + g as f32 * inv_alpha) as u8;
            let out_b = (pixel[2] as f32 * alpha + b as f32 * inv_alpha) as u8;

            result.put_pixel(x, y, image::Rgba([out_r, out_g, out_b, 255]));
        }
    }

    result
}
