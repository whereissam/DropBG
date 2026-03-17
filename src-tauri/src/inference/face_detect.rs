use image::DynamicImage;
use ndarray::Array4;
use ort::session::Session;
use std::sync::{Arc, Mutex};

const FACE_MODEL_URL: &str =
    "https://huggingface.co/opencv/face_detection_yunet/resolve/main/face_detection_yunet_2023mar.onnx";
const FACE_MODEL_FILENAME: &str = "yunet_face_detect.onnx";
const INPUT_SIZE: u32 = 320;
const CONF_THRESHOLD: f32 = 0.7;

#[derive(Clone)]
pub struct FaceDetectState {
    pub session: Arc<Mutex<Option<Session>>>,
}

unsafe impl Send for FaceDetectState {}
unsafe impl Sync for FaceDetectState {}

impl FaceDetectState {
    pub fn new() -> Self {
        Self {
            session: Arc::new(Mutex::new(None)),
        }
    }

    pub fn ensure_loaded(&self) -> Result<(), String> {
        let mut guard = self
            .session
            .lock()
            .map_err(|e| format!("Face detect lock poisoned: {e}"))?;
        if guard.is_none() {
            let path = face_model_path().map_err(|e| e.to_string())?;
            if !path.exists() {
                return Err("Face detection model not available".into());
            }

            let session = Session::builder()
                .map_err(|e| format!("Failed to create session builder: {e}"))?
                .commit_from_file(&path)
                .map_err(|e| format!("Failed to load face detection model: {e}"))?;

            *guard = Some(session);
        }
        Ok(())
    }
}

pub fn face_model_path() -> anyhow::Result<std::path::PathBuf> {
    let config = crate::model::downloader::load_config()?;
    Ok(std::path::PathBuf::from(&config.model_dir).join(FACE_MODEL_FILENAME))
}

pub fn face_model_exists() -> bool {
    face_model_path().map_or(false, |p| p.exists())
}

/// Download the YuNet face detection model (~233 KB).
pub fn download_face_model() -> anyhow::Result<()> {
    let dest = face_model_path()?;
    if dest.exists() {
        return Ok(());
    }
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let client = reqwest::blocking::Client::new();
    let resp = client.get(FACE_MODEL_URL).send()?;
    if !resp.status().is_success() {
        anyhow::bail!("Failed to download face model: {}", resp.status());
    }
    let bytes = resp.bytes()?;
    std::fs::write(&dest, &bytes)?;
    Ok(())
}

/// Detect faces in an image. Returns the number of faces found.
pub fn detect_faces(state: &FaceDetectState, img: &DynamicImage) -> Result<usize, String> {
    let mut guard = state
        .session
        .lock()
        .map_err(|e| format!("Face detect lock poisoned: {e}"))?;
    let session = guard.as_mut().ok_or("Face detection session not loaded")?;

    let resized = img.resize_exact(
        INPUT_SIZE,
        INPUT_SIZE,
        image::imageops::FilterType::Triangle,
    );
    let rgb = resized.to_rgb8();

    // YuNet expects [1, 3, 320, 320] BGR, but RGB works too
    // No normalization needed — YuNet handles raw pixel values
    let mut tensor = Array4::<f32>::zeros((1, 3, INPUT_SIZE as usize, INPUT_SIZE as usize));
    for y in 0..INPUT_SIZE as usize {
        for x in 0..INPUT_SIZE as usize {
            let pixel = rgb.get_pixel(x as u32, y as u32);
            tensor[[0, 0, y, x]] = pixel[0] as f32;
            tensor[[0, 1, y, x]] = pixel[1] as f32;
            tensor[[0, 2, y, x]] = pixel[2] as f32;
        }
    }

    let input_value = ort::value::Value::from_array(tensor)
        .map_err(|e| format!("Failed to create face input tensor: {e}"))?;

    let outputs = session
        .run(ort::inputs![input_value])
        .map_err(|e| format!("Face detection inference failed: {e}"))?;

    // YuNet output: each detection is a 15-element row
    // [x, y, w, h, <10 landmarks>, confidence]
    let (shape, data) = outputs[0]
        .try_extract_tensor::<f32>()
        .map_err(|e| format!("Failed to extract face output: {e}"))?;

    let num_detections = if shape.len() >= 2 { shape[0] as usize } else { 0 };
    let cols = if shape.len() >= 2 { shape[1] as usize } else { 15 };

    let mut face_count = 0;
    for i in 0..num_detections {
        let conf_idx = i * cols + 14; // confidence is at index 14
        if conf_idx < data.len() && data[conf_idx] >= CONF_THRESHOLD {
            face_count += 1;
        }
    }

    Ok(face_count)
}
