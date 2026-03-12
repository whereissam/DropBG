import { useState } from "react";
import { saveImage, autoCrop, getOutputDir, setOutputDir } from "../tauri";

interface Props {
  originalPath: string | null;
  resultBase64: string;
  onReset: () => void;
  onUpdateResult?: (newBase64: string) => void;
}

export default function Toolbar({ originalPath, resultBase64, onReset, onUpdateResult }: Props) {
  const [cropping, setCropping] = useState(false);

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
      <button className="btn btn-secondary" onClick={handleAutoCrop} disabled={cropping}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <path d="M6.13 1L6 16a2 2 0 0 0 2 2h15" />
          <path d="M1 6.13L16 6a2 2 0 0 1 2 2v15" />
        </svg>
        {cropping ? "Cropping..." : "Auto-Crop"}
      </button>
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
