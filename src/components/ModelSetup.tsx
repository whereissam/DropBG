import { useEffect, useState } from "react";
import { getModelInfo, openPathInFinder, setModelDir, type ModelInfo } from "../tauri";
import logoSvg from "../assets/logo.svg";

interface Props {
  downloading: boolean;
  progress: number;
  error: string | null;
  onDownload: () => void;
}

export default function ModelSetup({ downloading, progress, error, onDownload }: Props) {
  const [info, setInfo] = useState<ModelInfo | null>(null);

  useEffect(() => {
    getModelInfo().then(setInfo).catch(() => {});
  }, []);

  async function handleChangePath() {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({ directory: true, title: "Choose model download location" });
      if (selected) {
        await setModelDir(selected as string);
        const updated = await getModelInfo();
        setInfo(updated);
      }
    } catch (e) {
      console.error("Failed to change path:", e);
    }
  }

  return (
    <div className="setup-overlay">
      <div className="setup-card">
        <div className="setup-icon">
          <img src={logoSvg} alt="DropBG" width="72" height="72" style={{ borderRadius: 16 }} />
        </div>

        <h1>Welcome to DropBG</h1>
        <p className="setup-subtitle">
          Local AI background remover — your images never leave your Mac.
        </p>

        <div className="setup-info">
          <div className="info-row">
            <span className="info-label">AI Model</span>
            <span className="info-value">{info?.name ?? "BiRefNet Lite (fp16 ONNX)"}</span>
          </div>
          <div className="info-row">
            <span className="info-label">Size</span>
            <span className="info-value">~200 MB</span>
          </div>
          <div className="info-row">
            <span className="info-label">File</span>
            <span className="info-value mono-sm">{info?.filename ?? "birefnet_lite_fp16.onnx"}</span>
          </div>
          <div className="info-row">
            <span className="info-label">Runs On</span>
            <span className="info-value">Apple Neural Engine / CPU</span>
          </div>
        </div>

        {/* Download location — editable */}
        <div className="setup-location">
          <div className="loc-header">
            <span className="loc-label">Download to</span>
            {!downloading && (
              <button className="loc-change" onClick={handleChangePath}>Change</button>
            )}
          </div>
          <div
            className="loc-path"
            onClick={() => info && openPathInFinder(info.model_dir).catch(() => {})}
            title="Click to open in Finder"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
            </svg>
            <span>{info ? info.model_dir : "Loading..."}</span>
          </div>
        </div>

        {error && (
          <div className="setup-error">
            <strong>Download failed</strong>
            <p>{error}</p>
          </div>
        )}

        {downloading ? (
          <div className="setup-progress">
            <div className="progress-bar">
              <div className="progress-fill" style={{ width: `${progress}%` }} />
            </div>
            <span className="progress-text">Downloading... {progress.toFixed(1)}%</span>
          </div>
        ) : (
          <button className="setup-btn" onClick={onDownload}>
            {error ? "Retry Download" : "Download & Get Started"}
          </button>
        )}

        <p className="setup-footer">
          One-time setup. After this, everything runs 100% offline.
        </p>
      </div>
    </div>
  );
}
