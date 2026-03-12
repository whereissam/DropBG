use image::{DynamicImage, RgbaImage};

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

/// Apply the mask output from the model onto the original image as an alpha channel.
/// `mask_data` is a flat Vec of shape [1, 1, 1024, 1024].
pub fn apply_mask(
    original: &DynamicImage,
    mask_data: &[f32],
    mask_size: u32,
    orig_w: u32,
    orig_h: u32,
) -> Result<DynamicImage, String> {
    // Build grayscale mask image
    let mut mask_img = image::GrayImage::new(mask_size, mask_size);
    for y in 0..mask_size {
        for x in 0..mask_size {
            let idx = (y * mask_size + x) as usize;
            let val = sigmoid(mask_data[idx]);
            mask_img.put_pixel(x, y, image::Luma([(val * 255.0) as u8]));
        }
    }

    // Resize mask to original dimensions
    let resized_mask = image::imageops::resize(
        &mask_img,
        orig_w,
        orig_h,
        image::imageops::FilterType::Triangle,
    );

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
