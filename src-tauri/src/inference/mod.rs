pub mod apple_vision;
pub mod cloud;
pub mod face_detect;
pub mod postprocess;
pub mod preprocess;
pub mod refine;
pub mod session;
pub mod upscale;

use ndarray::Array4;
use ort::session::Session;

pub fn run_inference(
    session: &mut Session,
    input: Array4<f32>,
) -> Result<Vec<f32>, String> {
    let input_value = ort::value::Value::from_array(input)
        .map_err(|e| format!("Failed to create input tensor: {e}"))?;

    let outputs = session
        .run(ort::inputs![input_value])
        .map_err(|e| format!("Inference failed: {e}"))?;

    let output = &outputs[0];
    let (_shape, data) = output
        .try_extract_tensor::<f32>()
        .map_err(|e| format!("Failed to extract output: {e}"))?;

    Ok(data.to_vec())
}
