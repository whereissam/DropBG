use image::DynamicImage;
use ndarray::Array4;

const MEAN: [f32; 3] = [0.485, 0.456, 0.406];
const STD: [f32; 3] = [0.229, 0.224, 0.225];

pub fn preprocess(img: &DynamicImage, input_size: u32) -> anyhow::Result<Array4<f32>> {
    let resized = img.resize_exact(input_size, input_size, image::imageops::FilterType::Triangle);
    let rgb = resized.to_rgb8();

    let size = input_size as usize;
    let mut tensor = Array4::<f32>::zeros((1, 3, size, size));

    for y in 0..size {
        for x in 0..size {
            let pixel = rgb.get_pixel(x as u32, y as u32);
            for c in 0..3 {
                let val = pixel[c] as f32 / 255.0;
                tensor[[0, c, y, x]] = (val - MEAN[c]) / STD[c];
            }
        }
    }

    Ok(tensor)
}
