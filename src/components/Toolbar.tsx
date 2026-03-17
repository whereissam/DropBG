import { useState } from "react";
import { saveImage, autoCrop, upscaleImage, refineResult, getUpscaleModelInfo, getRefineModelInfo, getOutputDir, setOutputDir } from "../tauri";

interface Props {
  originalPath: string | null;
  resultBase64: string;
  onReset: () => void;
  onUpdateResult?: (newBase64: string) => void;
  onToast?: (msg: string, type: "success" | "error" | "info") => void;
}

export default function Toolbar({ originalPath, resultBase64, onReset, onUpdateResult, onToast }: Props) {
  const [cropping, setCropping] = useState(false);
  const [upscaling, setUpscaling] = useState(false);
  const [refining, setRefining] = useState(false);
  const [showScaleMenu, setShowScaleMenu] = useState(false);

  async function handleSave() {
    try {
      const { save } = await import("@tauri-apps/plugin-dialog");
      const outputDir = await getOutputDir().catch(() => null);
      const defaultName = getDefaultName();
      const defaultPath = outputDir ? `${outputDir}/${defaultName}` : defaultName;

      const path = await save({
        defaultPath,
        filters: [{ name: "PNG Image", extensions: ["png"] }],
      });
      if (path) {
        await saveImage(resultBase64, path);
        const lastSlash = path.lastIndexOf("/");
        if (lastSlash > 0) {
          const chosenDir = path.substring(0, lastSlash);
          if (chosenDir !== outputDir) {
            await setOutputDir(chosenDir).catch(() => {});
          }
        }
      }
    } catch (e) {
      console.error("Save error:", e);
    }
  }

  async function handleAutoCrop() {
    if (!onUpdateResult) return;
    setCropping(true);
    try {
      const cropped = await autoCrop(resultBase64);
      onUpdateResult(cropped);
    } catch (e) {
      console.error("Auto-crop error:", e);
    } finally {
      setCropping(false);
    }
  }

  async function handleUpscale(scale: number) {
    if (!onUpdateResult) return;
    setShowScaleMenu(false);

    // Check if model is downloaded
    try {
      const info = await getUpscaleModelInfo();
      if (!info.exists) {
        onToast?.("Upscale model not downloaded. Download it in Settings first.", "info");
        return;
      }
    } catch {
      onToast?.("Failed to check upscale model status.", "error");
      return;
    }

    setUpscaling(true);
    try {
      const result = await upscaleImage(resultBase64, scale);
      onUpdateResult(result);
      onToast?.(`Image upscaled ${scale}x successfully!`, "success");
    } catch (e: any) {
      console.error("Upscale error:", e);
      onToast?.("Upscale failed: " + e.toString(), "error");
    } finally {
      setUpscaling(false);
    }
  }

  async function handleRefine() {
    if (!onUpdateResult || !originalPath) return;

    try {
      const info = await getRefineModelInfo();
      if (!info.exists) {
        onToast?.("Refinement model not downloaded. Download it in Settings first.", "info");
        return;
      }
    } catch {
      onToast?.("Failed to check refine model status.", "error");
      return;
    }

    setRefining(true);
    try {
      const refined = await refineResult(resultBase64, originalPath);
      onUpdateResult(refined);
      onToast?.("Alpha edges refined with ViTMatte!", "success");
    } catch (e: any) {
      console.error("Refine error:", e);
      onToast?.("Refine failed: " + e.toString(), "error");
    } finally {
      setRefining(false);
    }
  }

  function getDefaultName() {
    if (!originalPath) return "output_nobg.png";
    const parts = originalPath.split("/");
    const filename = parts[parts.length - 1];
    const name = filename.replace(/\.[^.]+$/, "");
    return `${name}_nobg.png`;
  }

  return (
    <div className="toolbar">
      <button className="btn btn-secondary" onClick={onReset}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <line x1="12" y1="5" x2="12" y2="19" />
          <line x1="5" y1="12" x2="19" y2="12" />
        </svg>
        New Image
      </button>
      <div className="spacer" />
      <button className="btn btn-secondary" onClick={handleAutoCrop} disabled={cropping || upscaling || refining}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <path d="M6.13 1L6 16a2 2 0 0 0 2 2h15" />
          <path d="M1 6.13L16 6a2 2 0 0 1 2 2v15" />
        </svg>
        {cropping ? "Cropping..." : "Auto-Crop"}
      </button>
      <button className="btn btn-secondary" onClick={handleRefine} disabled={refining || upscaling || cropping || !originalPath}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z" />
        </svg>
        {refining ? "Refining..." : "Refine"}
      </button>
      <div className="upscale-wrapper">
        <button
          className="btn btn-secondary"
          onClick={() => setShowScaleMenu((v) => !v)}
          disabled={upscaling}
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <polyline points="15 3 21 3 21 9" />
            <polyline points="9 21 3 21 3 15" />
            <line x1="21" y1="3" x2="14" y2="10" />
            <line x1="3" y1="21" x2="10" y2="14" />
          </svg>
          {upscaling ? "Upscaling..." : "Upscale"}
        </button>
        {showScaleMenu && !upscaling && (
          <div className="scale-menu">
            <button className="scale-option" onClick={() => handleUpscale(2)}>2x</button>
            <button className="scale-option" onClick={() => handleUpscale(4)}>4x</button>
          </div>
        )}
      </div>
      <button className="btn btn-primary" onClick={handleSave}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
          <polyline points="7 10 12 15 17 10" />
          <line x1="12" y1="15" x2="12" y2="3" />
        </svg>
        Save PNG
      </button>
    </div>
  );
}
