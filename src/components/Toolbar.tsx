import { saveImage, getOutputDir, setOutputDir } from "../tauri";

interface Props {
  originalPath: string | null;
  resultBase64: string;
  onReset: () => void;
}

export default function Toolbar({ originalPath, resultBase64, onReset }: Props) {
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
        // Remember the chosen folder as default for next time
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
