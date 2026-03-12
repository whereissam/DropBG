async function getInvoke() {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke;
}

export interface ModelInfo {
  name: string;
  filename: string;
  download_url: string;
  exists: boolean;
  size_bytes: number;
  model_dir: string;
  model_path: string;
  variant: string;       // "Lite" | "Full"
  approx_size: string;
  description: string;
  other_variant: string;
  other_name: string;
  other_exists: boolean;
  other_approx_size: string;
  other_description: string;
}

export async function checkModelReady(): Promise<boolean> {
  const invoke = await getInvoke();
  return invoke<boolean>("check_model_ready");
}

export async function getModelInfo(): Promise<ModelInfo> {
  const invoke = await getInvoke();
  return invoke<ModelInfo>("get_model_info");
}

export async function openPathInFinder(path: string): Promise<void> {
  const invoke = await getInvoke();
  return invoke<void>("open_path_in_finder", { path });
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

export async function saveImage(base64Data: string, savePath: string): Promise<void> {
  const invoke = await getInvoke();
  return invoke<void>("save_image", { base64Data, savePath });
}
