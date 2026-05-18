use base64::Engine;

#[tauri::command]
pub fn replace_background_color(
    base64_data: String,
    r: u8, g: u8, b: u8,
) -> Result<String, String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&base64_data)
        .map_err(|e| format!("Invalid base64: {e}"))?;
    let img = image::load_from_memory(&bytes)
        .map_err(|e| format!("Failed to decode image: {e}"))?;
    let rgba = img.to_rgba8();

    let result = crate::imaging::background::replace_with_color(&rgba, r, g, b);
    encode_rgba_to_base64(&result)
}

#[tauri::command]
pub fn replace_background_gradient(
    base64_data: String,
    r1: u8, g1: u8, b1: u8,
    r2: u8, g2: u8, b2: u8,
) -> Result<String, String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&base64_data)
        .map_err(|e| format!("Invalid base64: {e}"))?;
    let img = image::load_from_memory(&bytes)
        .map_err(|e| format!("Failed to decode image: {e}"))?;
    let rgba = img.to_rgba8();

    let result = crate::imaging::background::replace_with_gradient(&rgba, r1, g1, b1, r2, g2, b2);
    encode_rgba_to_base64(&result)
}

#[tauri::command]
pub fn replace_background_image(
    base64_data: String,
    bg_image_path: String,
) -> Result<String, String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&base64_data)
        .map_err(|e| format!("Invalid base64: {e}"))?;
    let fg = image::load_from_memory(&bytes)
        .map_err(|e| format!("Failed to decode foreground: {e}"))?;

    let bg = image::open(&bg_image_path)
        .map_err(|e| format!("Failed to open background image: {e}"))?;

    let result = crate::imaging::background::replace_with_image(&fg.to_rgba8(), &bg);
    encode_rgba_to_base64(&result)
}

#[tauri::command]
pub fn auto_crop(base64_data: String, padding: Option<u32>) -> Result<String, String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&base64_data)
        .map_err(|e| format!("Invalid base64: {e}"))?;

    let img = image::load_from_memory(&bytes).map_err(|e| format!("Failed to decode image: {e}"))?;
    let cropped = crate::inference::postprocess::autocrop(&img, padding.unwrap_or(4));

    let mut buf = Vec::new();
    cropped
        .write_to(
            &mut std::io::Cursor::new(&mut buf),
            image::ImageFormat::Png,
        )
        .map_err(|e| format!("Failed to encode PNG: {e}"))?;

    Ok(base64::engine::general_purpose::STANDARD.encode(&buf))
}

fn encode_rgba_to_base64(img: &image::RgbaImage) -> Result<String, String> {
    let dyn_img = image::DynamicImage::ImageRgba8(img.clone());
    let mut buf = Vec::new();
    dyn_img
        .write_to(
            &mut std::io::Cursor::new(&mut buf),
            image::ImageFormat::Png,
        )
        .map_err(|e| format!("Failed to encode PNG: {e}"))?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&buf))
}
