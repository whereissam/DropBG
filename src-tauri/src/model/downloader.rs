use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

// ===== Model Variants =====

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ModelVariant {
    Lite,
    Full,
    BEN2,
    RMBG2,
    MODNet,
    InSPyReNet,
    Portrait,
    General,
    Matting,
    Dynamic,
}

impl Default for ModelVariant {
    fn default() -> Self {
        ModelVariant::Lite
    }
}

impl ModelVariant {
    pub fn all() -> &'static [ModelVariant] {
        &[
            ModelVariant::Lite,
            ModelVariant::Full,
            ModelVariant::Portrait,
            ModelVariant::General,
            ModelVariant::Matting,
            ModelVariant::Dynamic,
            ModelVariant::BEN2,
            ModelVariant::RMBG2,
            ModelVariant::InSPyReNet,
            ModelVariant::MODNet,
        ]
    }

    pub fn name(&self) -> &str {
        match self {
            ModelVariant::Lite => "BiRefNet Lite",
            ModelVariant::Full => "BiRefNet Full",
            ModelVariant::Portrait => "BiRefNet Portrait",
            ModelVariant::General => "BiRefNet General",
            ModelVariant::Matting => "BiRefNet Matting",
            ModelVariant::Dynamic => "BiRefNet Dynamic",
            ModelVariant::BEN2 => "BEN2",
            ModelVariant::RMBG2 => "RMBG 2.0",
            ModelVariant::InSPyReNet => "InSPyReNet",
            ModelVariant::MODNet => "MODNet",
        }
    }

    pub fn filename(&self) -> &str {
        match self {
            ModelVariant::Lite => "birefnet_lite_fp16.onnx",
            ModelVariant::Full => "birefnet_full_fp16.onnx",
            ModelVariant::Portrait => "birefnet_portrait_fp16.onnx",
            ModelVariant::General => "birefnet_general_fp16.onnx",
            ModelVariant::Matting => "birefnet_lite_matting_fp16.onnx",
            ModelVariant::Dynamic => "birefnet_dynamic_fp16.onnx",
            ModelVariant::BEN2 => "ben2_fp16.onnx",
            ModelVariant::RMBG2 => "rmbg2_fp16.onnx",
            ModelVariant::InSPyReNet => "inspyrenet_fp16.onnx",
            ModelVariant::MODNet => "modnet_fp16.onnx",
        }
    }

    pub fn url(&self) -> &str {
        match self {
            ModelVariant::Lite => "https://huggingface.co/onnx-community/BiRefNet_lite-ONNX/resolve/main/onnx/model_fp16.onnx",
            ModelVariant::Full => "https://huggingface.co/onnx-community/BiRefNet-ONNX/resolve/main/onnx/model_fp16.onnx",
            ModelVariant::Portrait => "https://huggingface.co/onnx-community/BiRefNet-portrait-ONNX/resolve/main/onnx/model_fp16.onnx",
            ModelVariant::General => "https://huggingface.co/onnx-community/BiRefNet-general-epoch_244/resolve/main/onnx/model_fp16.onnx",
            // Matting has no pre-built ONNX — must be exported via scripts/export_matting_onnx.py
            ModelVariant::Matting => "",
            // Dynamic has no pre-built ONNX — must be exported via scripts/export_dynamic_onnx.py
            ModelVariant::Dynamic => "",
            ModelVariant::BEN2 => "https://huggingface.co/onnx-community/BEN2-ONNX/resolve/main/onnx/model_fp16.onnx",
            ModelVariant::RMBG2 => "https://huggingface.co/briaai/RMBG-2.0/resolve/main/onnx/model_fp16.onnx",
            ModelVariant::InSPyReNet => "https://huggingface.co/OS-Software/InSPyReNet-SwinB-Plus-Ultra-ONNX/resolve/main/onnx/model_fp16.onnx",
            ModelVariant::MODNet => "https://huggingface.co/Xenova/modnet/resolve/main/onnx/model_fp16.onnx",
        }
    }

    /// URL for users to manually download gated models from HuggingFace web UI.
    pub fn manual_download_url(&self) -> Option<&str> {
        match self {
            ModelVariant::RMBG2 => Some("https://huggingface.co/briaai/RMBG-2.0/blob/main/onnx/model_fp16.onnx"),
            ModelVariant::Matting => Some("https://huggingface.co/ZhengPeng7/BiRefNet_lite-matting"),
            ModelVariant::Dynamic => Some("https://huggingface.co/ZhengPeng7/BiRefNet_dynamic"),
            ModelVariant::InSPyReNet => None,
            _ => None,
        }
    }

    /// Whether this model requires manual download (gated or needs ONNX export).
    pub fn requires_manual_download(&self) -> bool {
        matches!(self, ModelVariant::RMBG2 | ModelVariant::Matting | ModelVariant::Dynamic)
    }

    pub fn approx_size(&self) -> &str {
        match self {
            ModelVariant::Lite => "~200 MB",
            ModelVariant::Full => "~900 MB",
            ModelVariant::Portrait => "~490 MB",
            ModelVariant::General => "~490 MB",
            ModelVariant::Matting => "~214 MB",
            ModelVariant::Dynamic => "~490 MB",
            ModelVariant::BEN2 => "~219 MB",
            ModelVariant::RMBG2 => "~514 MB",
            ModelVariant::InSPyReNet => "~300 MB",
            ModelVariant::MODNet => "~13 MB",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            ModelVariant::Lite => "Fast, good for most images",
            ModelVariant::Full => "High quality BiRefNet, handles complex backgrounds",
            ModelVariant::Portrait => "Best for faces & people, specialized portrait model",
            ModelVariant::General => "Newer training (epoch 244), improved general quality",
            ModelVariant::Matting => "True alpha mattes for hair/fur/glass (export required)",
            ModelVariant::Dynamic => "Native resolution 256-2304px, no resize artifacts (export required)",
            ModelVariant::BEN2 => "Best on hair & fine edges, handles complex scenes",
            ModelVariant::RMBG2 => "BRIA's enhanced BiRefNet, excellent quality (manual download)",
            ModelVariant::InSPyReNet => "Excellent on fuzzy edges, hair strands & fine detail",
            ModelVariant::MODNet => "Lightweight, optimized for portraits (legacy)",
        }
    }

    /// Returns the fixed input size, or 0 for dynamic resolution models.
    pub fn input_size(&self) -> u32 {
        match self {
            ModelVariant::MODNet => 512,
            ModelVariant::InSPyReNet => 1024,
            ModelVariant::Dynamic => 0, // native resolution
            _ => 1024,
        }
    }

    /// Whether this model supports dynamic (native) resolution input.
    #[allow(dead_code)]
    pub fn is_dynamic(&self) -> bool {
        matches!(self, ModelVariant::Dynamic)
    }

    /// Whether this variant outputs true alpha mattes (not binary masks).
    #[allow(dead_code)]
    pub fn is_matting_model(&self) -> bool {
        matches!(self, ModelVariant::Matting)
    }

    /// Whether this variant is recommended for portrait/people images.
    #[allow(dead_code)]
    pub fn is_portrait_model(&self) -> bool {
        matches!(self, ModelVariant::Portrait)
    }

    pub fn variant_key(&self) -> &str {
        match self {
            ModelVariant::Lite => "Lite",
            ModelVariant::Full => "Full",
            ModelVariant::Portrait => "Portrait",
            ModelVariant::General => "General",
            ModelVariant::Matting => "Matting",
            ModelVariant::Dynamic => "Dynamic",
            ModelVariant::BEN2 => "BEN2",
            ModelVariant::RMBG2 => "RMBG2",
            ModelVariant::InSPyReNet => "InSPyReNet",
            ModelVariant::MODNet => "MODNet",
        }
    }

    pub fn from_key(key: &str) -> Option<ModelVariant> {
        match key {
            "Lite" => Some(ModelVariant::Lite),
            "Full" => Some(ModelVariant::Full),
            "Portrait" => Some(ModelVariant::Portrait),
            "General" => Some(ModelVariant::General),
            "Matting" => Some(ModelVariant::Matting),
            "Dynamic" => Some(ModelVariant::Dynamic),
            "BEN2" => Some(ModelVariant::BEN2),
            "RMBG2" => Some(ModelVariant::RMBG2),
            "InSPyReNet" => Some(ModelVariant::InSPyReNet),
            "MODNet" => Some(ModelVariant::MODNet),
            _ => None,
        }
    }
}

// ===== Upscale Model =====

const UPSCALE_MODEL_URL: &str =
    "https://huggingface.co/Xenova/realesrgan-x4plus/resolve/main/onnx/model.onnx";
const UPSCALE_MODEL_FILENAME: &str = "realesrgan_x4plus.onnx";

pub fn upscale_model_path() -> anyhow::Result<PathBuf> {
    let config = load_config()?;
    Ok(PathBuf::from(&config.model_dir).join(UPSCALE_MODEL_FILENAME))
}

pub fn upscale_model_exists() -> bool {
    upscale_model_path().map_or(false, |p| p.exists())
}

pub fn upscale_model_info() -> anyhow::Result<UpscaleModelInfo> {
    let path = upscale_model_path()?;
    let exists = path.exists();
    let size_bytes = if exists {
        std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };
    Ok(UpscaleModelInfo {
        name: "Real-ESRGAN x4plus".to_string(),
        filename: UPSCALE_MODEL_FILENAME.to_string(),
        exists,
        size_bytes,
        approx_size: "~64 MB".to_string(),
    })
}

#[derive(Serialize, Clone)]
pub struct UpscaleModelInfo {
    pub name: String,
    pub filename: String,
    pub exists: bool,
    pub size_bytes: u64,
    pub approx_size: String,
}

pub fn download_upscale_model<F>(on_progress: F) -> anyhow::Result<()>
where
    F: Fn(f64) + Send + 'static,
{
    let dest = upscale_model_path()?;
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }

    let client = reqwest::blocking::Client::new();
    let mut resp = client.get(UPSCALE_MODEL_URL).send()?;

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
            on_progress((downloaded as f64 / total_size as f64) * 100.0);
        }
    }

    file.flush()?;
    drop(file);
    fs::rename(&tmp_path, &dest)?;
    Ok(())
}

// ===== Cloud Provider =====

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum CloudProvider {
    Replicate,
    FalAI,
    RemoveBg,
}

impl CloudProvider {
    pub fn name(&self) -> &str {
        match self {
            CloudProvider::Replicate => "Replicate",
            CloudProvider::FalAI => "fal.ai",
            CloudProvider::RemoveBg => "remove.bg",
        }
    }

    pub fn all() -> &'static [CloudProvider] {
        &[CloudProvider::Replicate, CloudProvider::FalAI, CloudProvider::RemoveBg]
    }

    pub fn variant_key(&self) -> &str {
        match self {
            CloudProvider::Replicate => "Replicate",
            CloudProvider::FalAI => "FalAI",
            CloudProvider::RemoveBg => "RemoveBg",
        }
    }

    pub fn from_key(key: &str) -> Option<CloudProvider> {
        match key {
            "Replicate" => Some(CloudProvider::Replicate),
            "FalAI" => Some(CloudProvider::FalAI),
            "RemoveBg" => Some(CloudProvider::RemoveBg),
            _ => None,
        }
    }

    pub fn description(&self) -> &str {
        match self {
            CloudProvider::Replicate => "Cheapest (~$0.0004/img), runs BiRefNet on cloud GPUs",
            CloudProvider::FalAI => "Fast & reliable (~$0.018/img), RMBG 2.0 + BiRefNet",
            CloudProvider::RemoveBg => "Best polish (~$0.10/img), proprietary model",
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
    #[serde(default)]
    pub onboarding_done: bool,
    #[serde(default)]
    pub auto_model_routing: bool,
    #[serde(default)]
    pub cloud_enabled: bool,
    #[serde(default = "default_cloud_provider")]
    pub cloud_provider: CloudProvider,
    #[serde(default)]
    pub cloud_api_key: String,
}

fn default_cloud_provider() -> CloudProvider {
    CloudProvider::Replicate
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            model_dir: default_model_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
            output_dir: default_output_dir_string(),
            model_variant: ModelVariant::default(),
            onboarding_done: false,
            auto_model_routing: false,
            cloud_enabled: false,
            cloud_provider: default_cloud_provider(),
            cloud_api_key: String::new(),
        }
    }
}

impl Default for CloudProvider {
    fn default() -> Self {
        CloudProvider::Replicate
    }
}

fn default_output_dir_string() -> String {
    default_output_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

#[derive(Serialize, Clone)]
pub struct AlternativeModel {
    pub variant: String,
    pub name: String,
    pub exists: bool,
    pub approx_size: String,
    pub description: String,
    pub manual_download: bool,
    pub manual_download_url: Option<String>,
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
    pub manual_download: bool,
    pub manual_download_url: Option<String>,
    pub expected_filename: String,
    pub alternatives: Vec<AlternativeModel>,
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
        Ok(AppConfig::default())
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

    let alternatives: Vec<AlternativeModel> = ModelVariant::all()
        .iter()
        .filter(|v| *v != variant)
        .map(|v| {
            let alt_path = dir.join(v.filename());
            AlternativeModel {
                variant: v.variant_key().to_string(),
                name: v.name().to_string(),
                exists: alt_path.exists(),
                approx_size: v.approx_size().to_string(),
                description: v.description().to_string(),
                manual_download: v.requires_manual_download(),
                manual_download_url: v.manual_download_url().map(|s| s.to_string()),
            }
        })
        .collect();

    Ok(ModelInfo {
        name: variant.name().to_string(),
        filename: variant.filename().to_string(),
        download_url: variant.url().to_string(),
        exists,
        size_bytes,
        model_dir: dir.to_string_lossy().to_string(),
        model_path: path.to_string_lossy().to_string(),
        variant: variant.variant_key().to_string(),
        approx_size: variant.approx_size().to_string(),
        description: variant.description().to_string(),
        manual_download: variant.requires_manual_download(),
        manual_download_url: variant.manual_download_url().map(|s| s.to_string()),
        expected_filename: variant.filename().to_string(),
        alternatives,
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
