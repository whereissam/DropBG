import { useState } from "react";
import { saveImage, autoCrop, upscaleImage, refineResult, refineEdgesHr, decontaminateResult, getUpscaleModelInfo, getRefineModelInfo, getOutputDir, setOutputDir } from "../tauri";

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
  const [hrRefining, setHrRefining] = useState(false);
  const [decontaminating, setDecontaminating] = useState(false);
  const [showScaleMenu, setShowScaleMenu] = useState(false);
  const [showSaveMenu, setShowSaveMenu] = useState(false);

  async function doSave(dataB64: string) {
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
        await saveImage(dataB64, path);
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

  async function handleSave() {
    setShowSaveMenu(false);
    await doSave(resultBase64);
  }

  async function handleSave16Bit() {
    setShowSaveMenu(false);
    try {
      // 16-bit export carries the floating-point decontaminated color (no banding).
      const data = await decontaminateResult(resultBase64, true);
      await doSave(data);
    } catch (e: any) {
      onToast?.("16-bit export failed: " + e.toString(), "error");
    }
  }

  async function handleDecontaminate() {
    if (!onUpdateResult) return;
    setDecontaminating(true);
    try {
      const cleaned = await decontaminateResult(resultBase64, false);
      onUpdateResult(cleaned);
      onToast?.("Edge color decontaminated!", "success");
    } catch (e: any) {
      console.error("Decontaminate error:", e);
      onToast?.("Decontaminate failed: " + e.toString(), "error");
    } finally {
      setDecontaminating(false);
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

  async function handleRefineHr() {
    if (!onUpdateResult || !originalPath) return;
    setHrRefining(true);
    try {
      const refined = await refineEdgesHr(resultBase64, originalPath);
      onUpdateResult(refined);
      onToast?.("Edges refined with HR-matting!", "success");
    } catch (e: any) {
      console.error("HR refine error:", e);
      onToast?.(e.toString(), "error");
    } finally {
      setHrRefining(false);
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
      <button className="btn btn-secondary" onClick={handleRefine} disabled={refining || hrRefining || upscaling || cropping || !originalPath}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z" />
        </svg>
        {refining ? "Refining..." : "Refine"}
      </button>
      <button
        className="btn btn-secondary"
        onClick={handleRefineHr}
        disabled={refining || hrRefining || upscaling || cropping || !originalPath}
        title="Re-run HR-matting on just the soft edges (needs BiRefNet HR-matting downloaded)"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <path d="M3 7V5a2 2 0 0 1 2-2h2M17 3h2a2 2 0 0 1 2 2v2M21 17v2a2 2 0 0 1-2 2h-2M7 21H5a2 2 0 0 1-2-2v-2" />
          <circle cx="12" cy="12" r="3" />
        </svg>
        {hrRefining ? "Refining edges..." : "HR Edges"}
      </button>
      <button
        className="btn btn-secondary"
        onClick={handleDecontaminate}
        disabled={refining || hrRefining || decontaminating || upscaling || cropping || !onUpdateResult}
        title="Remove colored edge fringe by estimating the true foreground color"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <path d="M19 11l-7 7-7-7" />
          <path d="M12 2v16" />
          <circle cx="12" cy="20" r="1.5" />
        </svg>
        {decontaminating ? "Cleaning..." : "Decontaminate"}
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
      <div className="upscale-wrapper">
        <button className="btn btn-primary" onClick={() => setShowSaveMenu((v) => !v)}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
            <polyline points="7 10 12 15 17 10" />
            <line x1="12" y1="15" x2="12" y2="3" />
          </svg>
          Save PNG
        </button>
        {showSaveMenu && (
          <div className="scale-menu">
            <button className="scale-option" onClick={handleSave}>8-bit</button>
            <button className="scale-option" onClick={handleSave16Bit}>16-bit</button>
          </div>
        )}
      </div>
    </div>
  );
}
