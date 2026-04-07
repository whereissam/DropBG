use crate::model::downloader::{CloudProvider, load_config};
use base64::Engine;
use serde::Deserialize;

/// Remove background using a cloud API provider.
/// Takes image bytes, returns PNG bytes with transparent background.
pub fn remove_background_cloud(image_bytes: &[u8]) -> Result<Vec<u8>, String> {
    let config = load_config().map_err(|e| e.to_string())?;
    if config.cloud_api_key.is_empty() {
        return Err("No API key configured. Add your key in Settings → Cloud API.".to_string());
    }

    match config.cloud_provider {
        CloudProvider::Replicate => replicate_remove_bg(image_bytes, &config.cloud_api_key),
        CloudProvider::FalAI => fal_remove_bg(image_bytes, &config.cloud_api_key),
        CloudProvider::RemoveBg => removebg_remove_bg(image_bytes, &config.cloud_api_key),
    }
}

// ===== Replicate =====

#[derive(Deserialize)]
struct ReplicateCreateResponse {
    id: String,
    status: String,
    output: Option<serde_json::Value>,
    error: Option<String>,
    urls: Option<ReplicateUrls>,
}

#[derive(Deserialize)]
struct ReplicateUrls {
    get: String,
}

fn replicate_remove_bg(image_bytes: &[u8], api_key: &str) -> Result<Vec<u8>, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))?;

    let b64 = base64::engine::general_purpose::STANDARD.encode(image_bytes);
    let mime = if image_bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        "image/png"
    } else {
        "image/jpeg"
    };
    let data_uri = format!("data:{};base64,{}", mime, b64);

    // Use BiRefNet model on Replicate (most popular, 5.3M runs)
    let body = serde_json::json!({
        "version": "2af2eb1d1cd8b5a4e0968f4bb86f3696a2447ac3e94ed081e11bf14c87e3b0a6",
        "input": {
            "image": data_uri
        }
    });

    let resp = client
        .post("https://api.replicate.com/v1/predictions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .header("Prefer", "wait")
        .json(&body)
        .send()
        .map_err(|e| format!("Replicate API error: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().unwrap_or_default();
        return Err(format!("Replicate API error ({}): {}", status, text));
    }

    let result: ReplicateCreateResponse = resp.json()
        .map_err(|e| format!("Failed to parse Replicate response: {e}"))?;

    if let Some(err) = result.error {
        return Err(format!("Replicate error: {}", err));
    }

    // If status is "succeeded" and we have output, grab it
    if result.status == "succeeded" {
        return download_replicate_output(&client, &result.output);
    }

    // Otherwise poll until done
    let poll_url = result.urls
        .map(|u| u.get)
        .unwrap_or_else(|| format!("https://api.replicate.com/v1/predictions/{}", result.id));

    for _ in 0..60 {
        std::thread::sleep(std::time::Duration::from_secs(2));

        let poll_resp = client
            .get(&poll_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .map_err(|e| format!("Replicate poll error: {e}"))?;

        let poll: ReplicateCreateResponse = poll_resp.json()
            .map_err(|e| format!("Failed to parse poll response: {e}"))?;

        if let Some(err) = poll.error {
            return Err(format!("Replicate error: {}", err));
        }

        match poll.status.as_str() {
            "succeeded" => return download_replicate_output(&client, &poll.output),

            "failed" | "canceled" => return Err(format!("Replicate prediction {}", poll.status)),
            _ => continue, // starting, processing
        }
    }

    Err("Replicate prediction timed out after 120s".to_string())
}

fn download_replicate_output(
    client: &reqwest::blocking::Client,
    output: &Option<serde_json::Value>,
) -> Result<Vec<u8>, String> {
    let output = output.as_ref().ok_or("No output from Replicate")?;

    // Output can be a string URL or an array with one URL
    let url = if let Some(url) = output.as_str() {
        url.to_string()
    } else if let Some(arr) = output.as_array() {
        arr.first()
            .and_then(|v| v.as_str())
            .ok_or("Empty output array from Replicate")?
            .to_string()
    } else {
        return Err(format!("Unexpected Replicate output format: {}", output));
    };

    // Download the result image
    let img_resp = client
        .get(&url)
        .send()
        .map_err(|e| format!("Failed to download result: {e}"))?;

    let bytes = img_resp.bytes().map_err(|e| format!("Failed to read result: {e}"))?;
    Ok(bytes.to_vec())
}

// ===== fal.ai =====

fn fal_remove_bg(image_bytes: &[u8], api_key: &str) -> Result<Vec<u8>, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))?;

    let b64 = base64::engine::general_purpose::STANDARD.encode(image_bytes);
    let mime = if image_bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        "image/png"
    } else {
        "image/jpeg"
    };
    let data_uri = format!("data:{};base64,{}", mime, b64);

    // Use fal.ai BiRefNet endpoint
    let body = serde_json::json!({
        "image_url": data_uri
    });

    let resp = client
        .post("https://fal.run/fal-ai/birefnet")
        .header("Authorization", format!("Key {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .map_err(|e| format!("fal.ai API error: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().unwrap_or_default();
        return Err(format!("fal.ai API error ({}): {}", status, text));
    }

    #[derive(Deserialize)]
    struct FalImage {
        url: String,
    }
    #[derive(Deserialize)]
    struct FalResponse {
        image: FalImage,
    }

    let result: FalResponse = resp.json()
        .map_err(|e| format!("Failed to parse fal.ai response: {e}"))?;

    // Download result
    let img_resp = client
        .get(&result.image.url)
        .send()
        .map_err(|e| format!("Failed to download fal.ai result: {e}"))?;

    let bytes = img_resp.bytes().map_err(|e| format!("Failed to read result: {e}"))?;
    Ok(bytes.to_vec())
}

// ===== remove.bg =====

fn removebg_remove_bg(image_bytes: &[u8], api_key: &str) -> Result<Vec<u8>, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))?;

    let part = reqwest::blocking::multipart::Part::bytes(image_bytes.to_vec())
        .file_name("image.png")
        .mime_str("image/png")
        .map_err(|e| format!("Multipart error: {e}"))?;

    let form = reqwest::blocking::multipart::Form::new()
        .part("image_file", part)
        .text("size", "auto")
        .text("format", "png");

    let resp = client
        .post("https://api.remove.bg/v1.0/removebg")
        .header("X-Api-Key", api_key)
        .multipart(form)
        .send()
        .map_err(|e| format!("remove.bg API error: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().unwrap_or_default();
        return Err(format!("remove.bg API error ({}): {}", status, text));
    }

    let bytes = resp.bytes().map_err(|e| format!("Failed to read result: {e}"))?;
    Ok(bytes.to_vec())
}
