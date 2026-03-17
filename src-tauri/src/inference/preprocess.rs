use image::DynamicImage;
use ndarray::Array4;

const MEAN: [f32; 3] = [0.485, 0.456, 0.406];
const STD: [f32; 3] = [0.229, 0.224, 0.225];

/// Preprocess an image for inference.
/// If `input_size` is 0, uses the image's native resolution (rounded to 32px).
/// Otherwise resizes to input_size x input_size.
pub fn preprocess(img: &DynamicImage, input_size: u32) -> anyhow::Result<Array4<f32>> {
    let (target_w, target_h) = if input_size == 0 {
        // Dynamic resolution: round to nearest multiple of 32, clamp to 256-2304
        let w = ((img.width() + 15) / 32 * 32).clamp(256, 2304);
        let h = ((img.height() + 15) / 32 * 32).clamp(256, 2304);
        (w, h)
    } else {
        (input_size, input_size)
    };

    let resized = img.resize_exact(target_w, target_h, image::imageops::FilterType::Triangle);
    let rgb = resized.to_rgb8();

    let mut tensor = Array4::<f32>::zeros((1, 3, target_h as usize, target_w as usize));

    for y in 0..target_h as usize {
        for x in 0..target_w as usize {
            let pixel = rgb.get_pixel(x as u32, y as u32);
            for c in 0..3 {
                let val = pixel[c] as f32 / 255.0;
                tensor[[0, c, y, x]] = (val - MEAN[c]) / STD[c];
            }
        }
    }

    Ok(tensor)
}

/// Returns the actual mask dimensions after preprocessing.
/// For dynamic models (input_size=0), this depends on the image dimensions.
pub fn resolve_mask_size(img: &DynamicImage, input_size: u32) -> (u32, u32) {
    if input_size == 0 {
        let w = ((img.width() + 15) / 32 * 32).clamp(256, 2304);
        let h = ((img.height() + 15) / 32 * 32).clamp(256, 2304);
        (w, h)
    } else {
        (input_size, input_size)
    }
}
