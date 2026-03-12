import { useEffect, useState } from "react";
import {
  getModelInfo,
  getOutputDir,
  openPathInFinder,
  setModelDir,
  setOutputDir,
  deleteModel,
  type ModelInfo,
} from "../tauri";

interface Props {
  onClose: () => void;
  onModelDeleted: () => void;
  onToast: (msg: string, type: "success" | "error" | "info") => void;
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "—";
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export default function Settings({ onClose, onModelDeleted, onToast }: Props) {
  const [info, setInfo] = useState<ModelInfo | null>(null);
  const [outputDir, setOutputDirState] = useState<string | null>(null);
  const [confirming, setConfirming] = useState(false);

  useEffect(() => {
    loadInfo();
    getOutputDir().then(setOutputDirState).catch(() => {});
  }, []);

  async function loadInfo() {
    try {
      const i = await getModelInfo();
      setInfo(i);
    } catch (e) {
      console.error(e);
    }
  }

  async function handleOpenModelFolder() {
    if (!info) return;
    try {
      await openPathInFinder(info.exists ? info.model_path : info.model_dir);
    } catch (e: any) {
      onToast(e.toString(), "error");
    }
  }

  async function handleChangeModelDir() {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({ directory: true, title: "Choose model storage location" });
      if (selected) {
        const dir = selected as string;
        await setModelDir(dir);
        onToast("Model directory updated. Re-download to apply.", "info");
        await loadInfo();
      }
    } catch (e: any) {
      onToast(e.toString(), "error");
    }
  }

  async function handleChangeOutputDir() {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({ directory: true, title: "Choose default save location" });
      if (selected) {
        const dir = selected as string;
        await setOutputDir(dir);
        setOutputDirState(dir);
        onToast("Default save location updated.", "success");
      }
    } catch (e: any) {
      onToast(e.toString(), "error");
    }
  }

  async function handleOpenOutputFolder() {
    if (!outputDir) return;
    try {
      await openPathInFinder(outputDir);
    } catch (e: any) {
      onToast(e.toString(), "error");
    }
  }

  async function handleDelete() {
    if (!confirming) {
      setConfirming(true);
      return;
    }
    try {
      await deleteModel();
      onToast("Model deleted.", "info");
      onModelDeleted();
      onClose();
    } catch (e: any) {
      onToast(e.toString(), "error");
    }
  }

  return (
    <div className="settings-backdrop" onClick={onClose}>
      <div className="settings-panel" onClick={(e) => e.stopPropagation()}>
        <div className="settings-header">
          <h2>Settings</h2>
          <button className="settings-close" onClick={onClose}>×</button>
        </div>

        {/* ===== AI Model Section ===== */}
        <div className="settings-section">
          <h3>AI Model</h3>
          {info ? (
            <div className="settings-info">
              <div className="si-row">
                <span className="si-label">Name</span>
                <span className="si-value">{info.name}</span>
              </div>
              <div className="si-row">
                <span className="si-label">Status</span>
                <span className={`si-value ${info.exists ? "text-green" : "text-yellow"}`}>
                  {info.exists ? `Downloaded (${formatBytes(info.size_bytes)})` : "Not downloaded"}
                </span>
              </div>
              <div className="si-row">
                <span className="si-label">Location</span>
                <span
                  className="si-value mono clickable"
                  onClick={handleOpenModelFolder}
                  title="Open in Finder"
                >
                  {shortenPath(info.model_dir)}
                  <ExternalLinkIcon />
                </span>
              </div>
            </div>
          ) : (
            <p className="settings-loading">Loading...</p>
          )}

          <div className="settings-actions">
            <button className="sa-btn" onClick={handleChangeModelDir}>
              <FolderIcon />
              Change Model Location
            </button>
            {info && !info.exists && (
              <button
                className="sa-btn sa-btn-accent"
                onClick={() => { onModelDeleted(); onClose(); }}
              >
                <DownloadIcon />
                Download Model
              </button>
            )}
            {info?.exists && (
              <button
                className={`sa-btn sa-btn-danger ${confirming ? "confirming" : ""}`}
                onClick={handleDelete}
                onBlur={() => setConfirming(false)}
              >
                <TrashIcon />
                {confirming ? "Click again to confirm" : "Delete Model"}
              </button>
            )}
          </div>
        </div>

        {/* ===== Output Section ===== */}
        <div className="settings-section">
          <h3>Save Location</h3>
          <div className="settings-info">
            <div className="si-row">
              <span className="si-label">Default folder</span>
              <span
                className="si-value mono clickable"
                onClick={handleOpenOutputFolder}
                title="Open in Finder"
              >
                {outputDir ? shortenPath(outputDir) : "Loading..."}
                <ExternalLinkIcon />
              </span>
            </div>
          </div>
          <p className="settings-hint">
            Save dialog opens here by default. It also updates when you pick a different folder while saving.
          </p>
          <div className="settings-actions">
            <button className="sa-btn" onClick={handleChangeOutputDir}>
              <FolderIcon />
              Change Save Location
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

function shortenPath(p: string): string {
  const home = p.indexOf("/Users/");
  if (home >= 0) {
    const nextSlash = p.indexOf("/", home + 7);
    if (nextSlash > 0) return "~" + p.slice(nextSlash);
  }
  return p;
}

function ExternalLinkIcon() {
  return (
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ marginLeft: 4, verticalAlign: "middle" }}>
      <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
      <polyline points="15 3 21 3 21 9" />
      <line x1="10" y1="14" x2="21" y2="3" />
    </svg>
  );
}

function FolderIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
      <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
    </svg>
  );
}

function DownloadIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
      <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
      <polyline points="7 10 12 15 17 10" />
      <line x1="12" y1="15" x2="12" y2="3" />
    </svg>
  );
}

function TrashIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
      <polyline points="3 6 5 6 21 6" />
      <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
    </svg>
  );
}
