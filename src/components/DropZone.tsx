import { useEffect, useState } from "react";
import logoSvg from "../assets/logo.svg";

interface Props {
  ready: boolean;
  processing: boolean;
  error: string | null;
  onProcess: (filePath: string) => void;
  onReset: () => void;
}

const IMAGE_EXTS = ["png", "jpg", "jpeg", "webp", "bmp"];

function isImageFile(path: string) {
  const ext = path.split(".").pop()?.toLowerCase() ?? "";
  return IMAGE_EXTS.includes(ext);
}

export default function DropZone({ ready, processing, error, onProcess, onReset }: Props) {
  const [dragOver, setDragOver] = useState(false);
  const [progressStep, setProgressStep] = useState("");
  const [progressPercent, setProgressPercent] = useState(0);

  useEffect(() => {
    const cleanups: (() => void)[] = [];

    (async () => {
      try {
        const { listen } = await import("@tauri-apps/api/event");

        const u1 = await listen<{ paths: string[] }>("tauri://drag-drop", (e) => {
          setDragOver(false);
          if (!ready || !e.payload.paths?.length) return;
          const path = e.payload.paths[0];
          if (isImageFile(path)) onProcess(path);
        });
        cleanups.push(u1);

        const u2 = await listen("tauri://drag-enter", () => setDragOver(true));
        cleanups.push(u2);

        const u3 = await listen("tauri://drag-leave", () => setDragOver(false));
        cleanups.push(u3);

        const u4 = await listen<{ step: string; percent: number }>("process-progress", (e) => {
          setProgressStep(e.payload.step);
          setProgressPercent(e.payload.percent);
        });
        cleanups.push(u4);
      } catch (e) {
        console.warn("Tauri events not available:", e);
      }
    })();

    return () => cleanups.forEach((fn) => fn());
  }, [ready, onProcess]);

  // Reset progress when processing starts
  useEffect(() => {
    if (processing) {
      setProgressStep("Starting...");
      setProgressPercent(0);
    }
  }, [processing]);

  async function openFilePicker() {
    if (!ready) return;
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        multiple: false,
        filters: [{ name: "Images", extensions: IMAGE_EXTS }],
      });
      if (selected) onProcess(selected as string);
    } catch (e) {
      console.error("File picker error:", e);
    }
  }

  const zoneClass = [
    "dropzone",
    dragOver && "drag-over",
    !ready && "disabled",
  ].filter(Boolean).join(" ");

  return (
    <div className={zoneClass}>
      {processing ? (
        <div className="processing">
          <div className="spinner" />
          <p className="process-step">{progressStep}</p>
          <div className="process-progress">
            <div className="progress-bar wide">
              <div className="progress-fill" style={{ width: `${progressPercent}%` }} />
            </div>
            <span className="progress-text">{Math.round(progressPercent)}%</span>
          </div>
        </div>
      ) : error ? (
        <div className="error-state">
          <p>Error: {error}</p>
          <button onClick={onReset}>Try Again</button>
        </div>
      ) : (
        <div className="content" onClick={openFilePicker} role="button" tabIndex={0} onKeyDown={(e) => e.key === "Enter" && openFilePicker()}>
          <div className="icon">
            <img src={logoSvg} alt="DropBG" width="64" height="64" style={{ borderRadius: 14 }} />
          </div>
          <h2>Drop an image here</h2>
          <p>or click to browse</p>
          <p className="formats">PNG, JPEG, WebP supported</p>
          {!ready && <p className="warning">Waiting for model to download...</p>}
        </div>
      )}
    </div>
  );
}
