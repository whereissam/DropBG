use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

// ===== Model Variants =====

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ModelVariant {
    Lite,
    Full,
}

impl Default for ModelVariant {
    fn default() -> Self {
        ModelVariant::Lite
    }
}

impl ModelVariant {
    pub fn name(&self) -> &str {
        match self {
            ModelVariant::Lite => "BiRefNet Lite (fp16)",
            ModelVariant::Full => "BiRefNet Full (fp16)",
        }
    }

    pub fn filename(&self) -> &str {
        match self {
            ModelVariant::Lite => "birefnet_lite_fp16.onnx",
            ModelVariant::Full => "birefnet_full_fp16.onnx",
        }
    }

    pub fn url(&self) -> &str {
        match self {
            ModelVariant::Lite => "https://huggingface.co/onnx-community/BiRefNet_lite-ONNX/resolve/main/onnx/model_fp16.onnx",
            ModelVariant::Full => "https://huggingface.co/onnx-community/BiRefNet-ONNX/resolve/main/onnx/model_fp16.onnx",
        }
    }

    pub fn approx_size(&self) -> &str {
        match self {
            ModelVariant::Lite => "~200 MB",
            ModelVariant::Full => "~900 MB",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            ModelVariant::Lite => "Fast, good for most images",
            ModelVariant::Full => "Best quality, handles complex backgrounds",
        }
    }
}

// ===== Config =====

#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub model_dir: String,
    #[serde(default = "default_output_dir_string")]
    pub output_dir: String,
    #[serde(default)]
    pub model_variant: ModelVariant,
}

fn default_output_dir_string() -> String {
    default_output_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

#[derive(Serialize, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub filename: String,
    pub download_url: String,
    pub exists: bool,
    pub size_bytes: u64,
    pub model_dir: String,
    pub model_path: String,
    pub variant: String,
    pub approx_size: String,
    pub description: String,
    // Info about the other variant
    pub other_variant: String,
    pub other_name: String,
    pub other_exists: bool,
    pub other_approx_size: String,
    pub other_description: String,
}

fn config_path() -> anyhow::Result<PathBuf> {
    let base = dirs::data_dir().ok_or_else(|| anyhow::anyhow!("Cannot find data directory"))?;
    Ok(base.join("com.dropbg.app").join("config.json"))
}

fn default_model_dir() -> anyhow::Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
    Ok(home.join("Downloads").join("DropBG"))
}

fn default_output_dir() -> anyhow::Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
    Ok(home.join("Downloads"))
}

pub fn output_dir() -> anyhow::Result<PathBuf> {
    let config = load_config()?;
    Ok(PathBuf::from(&config.output_dir))
}

pub fn load_config() -> anyhow::Result<AppConfig> {
    let path = config_path()?;
    if path.exists() {
        let data = fs::read_to_string(&path)?;
        let config: AppConfig = serde_json::from_str(&data)?;
        Ok(config)
    } else {
        Ok(AppConfig {
            model_dir: default_model_dir()?.to_string_lossy().to_string(),
            output_dir: default_output_dir()?.to_string_lossy().to_string(),
            model_variant: ModelVariant::default(),
        })
    }
}

pub fn save_config(config: &AppConfig) -> anyhow::Result<()> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(config)?;
    fs::write(&path, data)?;
    Ok(())
}

pub fn current_variant() -> anyhow::Result<ModelVariant> {
    Ok(load_config()?.model_variant)
}

#[allow(dead_code)]
pub fn model_dir() -> anyhow::Result<PathBuf> {
    let config = load_config()?;
    Ok(PathBuf::from(&config.model_dir))
}

pub fn model_path() -> anyhow::Result<PathBuf> {
    let config = load_config()?;
    Ok(PathBuf::from(&config.model_dir).join(config.model_variant.filename()))
}

#[allow(dead_code)]
pub fn model_path_for_variant(variant: &ModelVariant) -> anyhow::Result<PathBuf> {
    let config = load_config()?;
    Ok(PathBuf::from(&config.model_dir).join(variant.filename()))
}

pub fn get_model_info() -> anyhow::Result<ModelInfo> {
    let config = load_config()?;
    let dir = PathBuf::from(&config.model_dir);
    let variant = &config.model_variant;
    let path = dir.join(variant.filename());
    let exists = path.exists();
    let size_bytes = if exists {
        fs::metadata(&path).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };

    let other = match variant {
        ModelVariant::Lite => ModelVariant::Full,
        ModelVariant::Full => ModelVariant::Lite,
    };
    let other_path = dir.join(other.filename());
    let other_exists = other_path.exists();

    Ok(ModelInfo {
        name: variant.name().to_string(),
        filename: variant.filename().to_string(),
        download_url: variant.url().to_string(),
        exists,
        size_bytes,
        model_dir: dir.to_string_lossy().to_string(),
        model_path: path.to_string_lossy().to_string(),
        variant: format!("{:?}", variant),
        approx_size: variant.approx_size().to_string(),
        description: variant.description().to_string(),
        other_variant: format!("{:?}", other),
        other_name: other.name().to_string(),
        other_exists,
        other_approx_size: other.approx_size().to_string(),
        other_description: other.description().to_string(),
    })
}

pub fn download_model_variant<F>(variant: &ModelVariant, dest: &PathBuf, on_progress: F) -> anyhow::Result<()>
where
    F: Fn(f64) + Send + 'static,
{
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }

    let client = reqwest::blocking::Client::new();
    let mut resp = client.get(variant.url()).send()?;

    if !resp.status().is_success() {
        anyhow::bail!("Download failed with status: {}", resp.status());
    }

    let total_size = resp.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    let tmp_path = dest.with_extension("onnx.tmp");
    let mut file = fs::File::create(&tmp_path)?;

    let mut buffer = [0u8; 8192];
    loop {
        let bytes_read = std::io::Read::read(&mut resp, &mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        file.write_all(&buffer[..bytes_read])?;
        downloaded += bytes_read as u64;

        if total_size > 0 {
            let progress = (downloaded as f64 / total_size as f64) * 100.0;
            on_progress(progress);
        }
    }

    file.flush()?;
    drop(file);

    fs::rename(&tmp_path, dest)?;

    Ok(())
}

// Keep backward compat — downloads the currently selected variant
pub fn download_model<F>(dest: &PathBuf, on_progress: F) -> anyhow::Result<()>
where
    F: Fn(f64) + Send + 'static,
{
    let variant = current_variant()?;
    download_model_variant(&variant, dest, on_progress)
}
