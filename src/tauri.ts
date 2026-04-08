async function getInvoke() {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke;
}

export interface AlternativeModel {
  variant: string;
  name: string;
  exists: boolean;
  approx_size: string;
  description: string;
  manual_download: boolean;
  manual_download_url: string | null;
}

export interface ModelInfo {
  name: string;
  filename: string;
  download_url: string;
  exists: boolean;
  size_bytes: number;
  model_dir: string;
  model_path: string;
  variant: string;
  approx_size: string;
  description: string;
  manual_download: boolean;
  manual_download_url: string | null;
  expected_filename: string;
  alternatives: AlternativeModel[];
}

export async function appleVisionAvailable(): Promise<boolean> {
  const invoke = await getInvoke();
  return invoke<boolean>("apple_vision_available");
}

export async function removeBackgroundAppleVision(imagePath: string): Promise<string> {
  const invoke = await getInvoke();
  return invoke<string>("remove_background_apple_vision", { imagePath });
}

export async function checkModelReady(): Promise<boolean> {
  const invoke = await getInvoke();
  return invoke<boolean>("check_model_ready");
}

export async function isOnboardingDone(): Promise<boolean> {
  const invoke = await getInvoke();
  return invoke<boolean>("is_onboarding_done");
}

export async function completeOnboarding(): Promise<void> {
  const invoke = await getInvoke();
  return invoke<void>("complete_onboarding");
}

export async function getAutoRouting(): Promise<boolean> {
  const invoke = await getInvoke();
  return invoke<boolean>("get_auto_routing");
}

export async function setAutoRouting(enabled: boolean): Promise<void> {
  const invoke = await getInvoke();
  return invoke<void>("set_auto_routing", { enabled });
}

export async function getModelInfo(): Promise<ModelInfo> {
  const invoke = await getInvoke();
  return invoke<ModelInfo>("get_model_info");
}

export async function openPathInFinder(path: string): Promise<void> {
  const invoke = await getInvoke();
  return invoke<void>("open_path_in_finder", { path });
}

export async function openUrlInBrowser(url: string): Promise<void> {
  const invoke = await getInvoke();
  return invoke<void>("open_url_in_browser", { url });
}

export async function getOutputDir(): Promise<string> {
  const invoke = await getInvoke();
  return invoke<string>("get_output_dir");
}

export async function setOutputDir(newDir: string): Promise<void> {
  const invoke = await getInvoke();
  return invoke<void>("set_output_dir", { newDir });
}

export async function setModelDir(newDir: string): Promise<void> {
  const invoke = await getInvoke();
  return invoke<void>("set_model_dir", { newDir });
}

export async function setModelVariant(variant: string): Promise<void> {
  const invoke = await getInvoke();
  return invoke<void>("set_model_variant", { variant });
}

export async function deleteModel(): Promise<void> {
  const invoke = await getInvoke();
  return invoke<void>("delete_model");
}

export async function downloadModel(): Promise<void> {
  const invoke = await getInvoke();
  return invoke<void>("download_model");
}

export async function removeBackground(imagePath: string): Promise<string> {
  const invoke = await getInvoke();
  return invoke<string>("remove_background", { imagePath });
}

export async function removeBackgroundBatch(
  imagePaths: string[],
  outputDir: string,
): Promise<string[]> {
  const invoke = await getInvoke();
  return invoke<string[]>("remove_background_batch", { imagePaths, outputDir });
}

export async function replaceBackgroundColor(
  base64Data: string, r: number, g: number, b: number,
): Promise<string> {
  const invoke = await getInvoke();
  return invoke<string>("replace_background_color", { base64Data, r, g, b });
}

export async function replaceBackgroundGradient(
  base64Data: string,
  r1: number, g1: number, b1: number,
  r2: number, g2: number, b2: number,
): Promise<string> {
  const invoke = await getInvoke();
  return invoke<string>("replace_background_gradient", { base64Data, r1, g1, b1, r2, g2, b2 });
}

export async function replaceBackgroundImage(
  base64Data: string, bgImagePath: string,
): Promise<string> {
  const invoke = await getInvoke();
  return invoke<string>("replace_background_image", { base64Data, bgImagePath });
}

export async function autoCrop(base64Data: string, padding?: number): Promise<string> {
  const invoke = await getInvoke();
  return invoke<string>("auto_crop", { base64Data, padding });
}

// ===== Upscale =====

export interface UpscaleModelInfo {
  name: string;
  filename: string;
  exists: boolean;
  size_bytes: number;
  approx_size: string;
}

export async function getUpscaleModelInfo(): Promise<UpscaleModelInfo> {
  const invoke = await getInvoke();
  return invoke<UpscaleModelInfo>("get_upscale_model_info");
}

export async function downloadUpscaleModel(): Promise<void> {
  const invoke = await getInvoke();
  return invoke<void>("download_upscale_model");
}

export async function upscaleImage(base64Data: string, scale?: number): Promise<string> {
  const invoke = await getInvoke();
  return invoke<string>("upscale_image", { base64Data, scale });
}

// ===== Refine (ViTMatte) =====

export interface RefineModelInfo {
  name: string;
  exists: boolean;
  size_bytes: number;
  approx_size: string;
}

export async function getRefineModelInfo(): Promise<RefineModelInfo> {
  const invoke = await getInvoke();
  return invoke<RefineModelInfo>("get_refine_model_info");
}

export async function downloadRefineModel(): Promise<void> {
  const invoke = await getInvoke();
  return invoke<void>("download_refine_model");
}

export async function refineResult(base64Data: string, originalPath: string): Promise<string> {
  const invoke = await getInvoke();
  return invoke<string>("refine_result", { base64Data, originalPath });
}

export async function saveImage(base64Data: string, savePath: string): Promise<void> {
  const invoke = await getInvoke();
  return invoke<void>("save_image", { base64Data, savePath });
}

// ===== Cloud API =====

export interface CloudProviderInfo {
  key: string;
  name: string;
  description: string;
}

export interface CloudConfig {
  enabled: boolean;
  provider: string;
  provider_name: string;
  has_api_key: boolean;
  providers: CloudProviderInfo[];
}

export async function getCloudConfig(): Promise<CloudConfig> {
  const invoke = await getInvoke();
  return invoke<CloudConfig>("get_cloud_config");
}

export async function setCloudEnabled(enabled: boolean): Promise<void> {
  const invoke = await getInvoke();
  return invoke<void>("set_cloud_enabled", { enabled });
}

export async function setCloudProvider(provider: string): Promise<void> {
  const invoke = await getInvoke();
  return invoke<void>("set_cloud_provider", { provider });
}

export async function setCloudApiKey(apiKey: string): Promise<void> {
  const invoke = await getInvoke();
  return invoke<void>("set_cloud_api_key", { apiKey });
}

export async function removeBackgroundCloud(imagePath: string): Promise<string> {
  const invoke = await getInvoke();
  return invoke<string>("remove_background_cloud", { imagePath });
}

export async function removeBackgroundBatchCloud(
  imagePaths: string[],
  outputDir: string,
): Promise<string[]> {
  const invoke = await getInvoke();
  return invoke<string[]>("remove_background_batch_cloud", { imagePaths, outputDir });
}
