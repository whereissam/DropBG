use base64::Engine;
use std::ffi::CString;

extern "C" {
    fn apple_vision_remove_background(
        path_ptr: *const i8,
        out_len: *mut usize,
    ) -> *mut u8;

    fn apple_vision_free_buffer(ptr: *mut u8, len: usize);

    fn apple_vision_available() -> bool;
}

/// Check if Apple Vision segmentation is available on this system.
pub fn is_available() -> bool {
    unsafe { apple_vision_available() }
}

/// Remove background using Apple Vision framework.
/// Returns base64-encoded PNG with alpha channel.
pub fn remove_background(image_path: &str) -> Result<String, String> {
    let c_path = CString::new(image_path)
        .map_err(|_| "Invalid path".to_string())?;

    let mut out_len: usize = 0;

    let ptr = unsafe {
        apple_vision_remove_background(c_path.as_ptr(), &mut out_len)
    };

    if ptr.is_null() || out_len == 0 {
        return Err("Apple Vision segmentation failed. The image may not contain recognizable subjects.".to_string());
    }

    let png_data = unsafe {
        let slice = std::slice::from_raw_parts(ptr, out_len);
        let data = slice.to_vec();
        apple_vision_free_buffer(ptr, out_len);
        data
    };

    Ok(base64::engine::general_purpose::STANDARD.encode(&png_data))
}
