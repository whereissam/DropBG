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

export async function saveImage(base64Data: string, savePath: string): Promise<void> {
  const invoke = await getInvoke();
  return invoke<void>("save_image", { base64Data, savePath });
}
