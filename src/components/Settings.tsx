import { useEffect, useState } from "react";
import {
  getModelInfo,
  getOutputDir,
  getAutoRouting,
  setAutoRouting,
  getUpscaleModelInfo,
  downloadUpscaleModel,
  getRefineModelInfo,
  downloadRefineModel,
  getCloudConfig,
  setCloudEnabled,
  setCloudProvider,
  setCloudApiKey,
  getCloudUsage,
  resetCloudUsage,
  openPathInFinder,
  openUrlInBrowser,
  setModelDir,
  setModelVariant,
  setOutputDir,
  deleteModel,
  type ModelInfo,
  type UpscaleModelInfo,
  type RefineModelInfo,
  type CloudConfig,
  type CloudUsage,
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
  const [switching, setSwitching] = useState(false);
  const [autoRouting, setAutoRoutingState] = useState(false);
  const [upscaleInfo, setUpscaleInfo] = useState<UpscaleModelInfo | null>(null);
  const [upscaleDownloading, setUpscaleDownloading] = useState(false);
  const [upscaleProgress, setUpscaleProgress] = useState(0);
  const [refineInfo, setRefineInfo] = useState<RefineModelInfo | null>(null);
  const [refineDownloading, setRefineDownloading] = useState(false);
  const [refineProgress, setRefineProgress] = useState(0);
  const [cloudConfig, setCloudConfig] = useState<CloudConfig | null>(null);
  const [cloudUsage, setCloudUsage] = useState<CloudUsage | null>(null);
  const [apiKeyInput, setApiKeyInput] = useState("");
  const [apiKeySaved, setApiKeySaved] = useState(false);

  useEffect(() => {
    loadInfo();
    getOutputDir().then(setOutputDirState).catch(() => {});
    getAutoRouting().then(setAutoRoutingState).catch(() => {});
    getRefineModelInfo().then(setRefineInfo).catch(() => {});
    getUpscaleModelInfo().then(setUpscaleInfo).catch(() => {});
    getCloudConfig().then(setCloudConfig).catch(() => {});
    getCloudUsage().then(setCloudUsage).catch(() => {});
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

  async function handleSwitchVariant(variantKey: string) {
    if (!info) return;
    setSwitching(true);
    try {
      await setModelVariant(variantKey);
      await loadInfo();
      const updated = await getModelInfo().catch(() => null);
      if (updated && !updated.exists) {
        onToast(`Switched to ${updated.name}. Download required.`, "info");
      } else {
        onToast(`Switched to ${updated?.name ?? variantKey}. Ready to use.`, "success");
      }
    } catch (e: any) {
      onToast(e.toString(), "error");
    } finally {
      setSwitching(false);
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

  async function handleDownloadUpscale() {
    setUpscaleDownloading(true);
    setUpscaleProgress(0);
    try {
      const { listen } = await import("@tauri-apps/api/event");
      const unlisten = await listen<number>("upscale-download-progress", (e) => {
        setUpscaleProgress(e.payload);
      });
      try {
        await downloadUpscaleModel();
        const updated = await getUpscaleModelInfo().catch(() => null);
        setUpscaleInfo(updated);
        onToast("Upscale model downloaded!", "success");
      } finally {
        unlisten();
      }
    } catch (e: any) {
      onToast("Upscale download failed: " + e.toString(), "error");
    } finally {
      setUpscaleDownloading(false);
    }
  }

  async function handleDownloadRefine() {
    setRefineDownloading(true);
    setRefineProgress(0);
    try {
      const { listen } = await import("@tauri-apps/api/event");
      const unlisten = await listen<number>("refine-download-progress", (e) => {
        setRefineProgress(e.payload);
      });
      try {
        await downloadRefineModel();
        const updated = await getRefineModelInfo().catch(() => null);
        setRefineInfo(updated);
        onToast("Refinement model downloaded!", "success");
      } finally {
        unlisten();
      }
    } catch (e: any) {
      onToast("Refine download failed: " + e.toString(), "error");
    } finally {
      setRefineDownloading(false);
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
            <>
              <div className="settings-info">
                <div className="si-row">
                  <span className="si-label">Active</span>
                  <span className="si-value">{info.name}</span>
                </div>
                <div className="si-row">
                  <span className="si-label">Quality</span>
                  <span className="si-value" style={{ fontSize: "0.75rem", color: "#999" }}>
                    {info.description}
                  </span>
                </div>
                <div className="si-row">
                  <span className="si-label">Status</span>
                  <span className={`si-value ${info.exists ? "text-green" : "text-yellow"}`}>
                    {info.exists ? `Downloaded (${formatBytes(info.size_bytes)})` : `Not downloaded (${info.approx_size})`}
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

              {/* Model variant switcher */}
              <div className="model-switcher">
                <div className="model-option active">
                  <div className="model-option-header">
                    <span className="model-option-dot active" />
                    <span className="model-option-name">{info.name}</span>
                  </div>
                  <span className="model-option-desc">{info.description}</span>
                </div>
                {info.alternatives.map((alt) => (
                  <div
                    key={alt.variant}
                    className={`model-option ${switching ? "switching" : "clickable"}`}
                    onClick={!switching ? () => handleSwitchVariant(alt.variant) : undefined}
                  >
                    <div className="model-option-header">
                      <span className="model-option-dot" />
                      <span className="model-option-name">{alt.name}</span>
                      {alt.exists && <span className="model-option-badge">Downloaded</span>}
                      {!alt.exists && alt.manual_download && (
                        <span className="model-option-badge needs-manual">{alt.approx_size}</span>
                      )}
                      {!alt.exists && !alt.manual_download && (
                        <span className="model-option-badge needs-dl">{alt.approx_size}</span>
                      )}
                    </div>
                    <span className="model-option-desc">
                      {switching ? "Switching..." : alt.description}
                    </span>
                  </div>
                ))}
              </div>

              {/* Auto-routing toggle */}
              <div className="auto-routing-toggle">
                <label className="toggle-row">
                  <div>
                    <strong>Auto model routing</strong>
                    <p>Detect faces and auto-switch to Portrait model</p>
                  </div>
                  <input
                    type="checkbox"
                    checked={autoRouting}
                    onChange={async (e) => {
                      const enabled = e.target.checked;
                      try {
                        await setAutoRouting(enabled);
                        setAutoRoutingState(enabled);
                        onToast(
                          enabled ? "Auto-routing enabled. Face detection model downloaded." : "Auto-routing disabled.",
                          "info",
                        );
                      } catch (err: any) {
                        onToast("Failed: " + err.toString(), "error");
                      }
                    }}
                  />
                </label>
              </div>
            </>
          ) : (
            <p className="settings-loading">Loading...</p>
          )}

          {info && !info.exists && info.manual_download && info.manual_download_url && (
            <div className="manual-download-hint">
              {(info.variant === "Matting" || info.variant === "Dynamic") ? (
                <>
                  <p>This model requires ONNX export (no pre-built ONNX available):</p>
                  <ol>
                    <li>Run <code>pip install torch transformers onnx onnxconverter-common</code></li>
                    <li>Run <code>python scripts/export_{info.variant === "Matting" ? "matting" : "dynamic"}_onnx.py</code></li>
                    <li>
                      Copy <code>{info.expected_filename}</code> to{" "}
                      <span className="clickable-link" onClick={handleOpenModelFolder}>
                        {shortenPath(info.model_dir)} <ExternalLinkIcon />
                      </span>
                    </li>
                  </ol>
                  <p style={{ marginTop: "0.4rem" }}>
                    <span
                      className="clickable-link"
                      onClick={() => openUrlInBrowser(info.manual_download_url!).catch(() => {})}
                    >
                      View source model on HuggingFace <ExternalLinkIcon />
                    </span>
                  </p>
                </>
              ) : (
                <>
                  <p>This model requires manual download from HuggingFace:</p>
                  <ol>
                    <li>
                      <span
                        className="clickable-link"
                        onClick={() => openUrlInBrowser(info.manual_download_url!).catch(() => {})}
                      >
                        Download from HuggingFace <ExternalLinkIcon />
                      </span>
                    </li>
                    <li>Rename the file to <code>{info.expected_filename}</code></li>
                    <li>
                      Place it in{" "}
                      <span className="clickable-link" onClick={handleOpenModelFolder}>
                        {shortenPath(info.model_dir)} <ExternalLinkIcon />
                      </span>
                    </li>
                  </ol>
                </>
              )}
            </div>
          )}
          <div className="settings-actions">
            <button className="sa-btn" onClick={handleChangeModelDir}>
              <FolderIcon />
              Change Model Location
            </button>
            {info && !info.exists && !info.manual_download && (
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

        {/* ===== Upscale Model Section ===== */}
        <div className="settings-section">
          <h3>AI Upscale</h3>
          {upscaleInfo ? (
            <div className="settings-info">
              <div className="si-row">
                <span className="si-label">Model</span>
                <span className="si-value">{upscaleInfo.name}</span>
              </div>
              <div className="si-row">
                <span className="si-label">Status</span>
                <span className={`si-value ${upscaleInfo.exists ? "text-green" : "text-yellow"}`}>
                  {upscaleInfo.exists
                    ? `Downloaded (${formatBytes(upscaleInfo.size_bytes)})`
                    : `Not downloaded (${upscaleInfo.approx_size})`}
                </span>
              </div>
            </div>
          ) : (
            <p className="settings-loading">Loading...</p>
          )}
          {upscaleDownloading && (
            <div className="upscale-progress">
              <div className="upscale-progress-bar" style={{ width: `${upscaleProgress}%` }} />
              <span className="upscale-progress-text">{upscaleProgress}%</span>
            </div>
          )}
          <p className="settings-hint">
            Optional model for AI-powered image upscaling (2x / 4x). Uses Real-ESRGAN.
          </p>
          {upscaleInfo && !upscaleInfo.exists && !upscaleDownloading && (
            <div className="settings-actions">
              <button className="sa-btn sa-btn-accent" onClick={handleDownloadUpscale}>
                <DownloadIcon />
                Download Upscale Model ({upscaleInfo.approx_size})
              </button>
            </div>
          )}
        </div>

        {/* ===== Refine Model Section ===== */}
        <div className="settings-section">
          <h3>Alpha Refinement</h3>
          {refineInfo ? (
            <div className="settings-info">
              <div className="si-row">
                <span className="si-label">Model</span>
                <span className="si-value">{refineInfo.name}</span>
              </div>
              <div className="si-row">
                <span className="si-label">Status</span>
                <span className={`si-value ${refineInfo.exists ? "text-green" : "text-yellow"}`}>
                  {refineInfo.exists
                    ? `Downloaded (${formatBytes(refineInfo.size_bytes)})`
                    : `Not downloaded (${refineInfo.approx_size})`}
                </span>
              </div>
            </div>
          ) : (
            <p className="settings-loading">Loading...</p>
          )}
          {refineDownloading && (
            <div className="upscale-progress">
              <div className="upscale-progress-bar" style={{ width: `${refineProgress}%` }} />
              <span className="upscale-progress-text">{refineProgress}%</span>
            </div>
          )}
          <p className="settings-hint">
            Two-stage pipeline: BiRefNet gives a coarse mask, then ViTMatte refines the edges for hair, fur, and transparency. Click "Refine" in the toolbar after removing the background.
          </p>
          {refineInfo && !refineInfo.exists && !refineDownloading && (
            <div className="settings-actions">
              <button className="sa-btn sa-btn-accent" onClick={handleDownloadRefine}>
                <DownloadIcon />
                Download Refine Model ({refineInfo.approx_size})
              </button>
            </div>
          )}
        </div>

        {/* ===== Cloud API Section ===== */}
        <div className="settings-section">
          <h3>Cloud API</h3>
          <p className="settings-hint" style={{ marginBottom: 8 }}>
            Use cloud GPU services for background removal. No local model download needed. Bring your own API key.
          </p>

          {cloudConfig && (
            <>
              {/* Enable toggle */}
              <div className="auto-routing-toggle">
                <label className="toggle-row">
                  <div>
                    <strong>Enable cloud processing</strong>
                    <p>Use cloud API instead of local models</p>
                  </div>
                  <input
                    type="checkbox"
                    checked={cloudConfig.enabled}
                    onChange={async (e) => {
                      const enabled = e.target.checked;
                      try {
                        await setCloudEnabled(enabled);
                        setCloudConfig({ ...cloudConfig, enabled });
                        onToast(enabled ? "Cloud mode enabled" : "Switched to local models", "info");
                      } catch (err: any) {
                        onToast("Failed: " + err.toString(), "error");
                      }
                    }}
                  />
                </label>
              </div>

              {cloudConfig.enabled && (
                <>
                  {/* Provider selector */}
                  <div className="model-switcher" style={{ marginTop: 8 }}>
                    {cloudConfig.providers.map((p) => (
                      <div
                        key={p.key}
                        className={`model-option ${p.key === cloudConfig.provider ? "active" : "clickable"}`}
                        onClick={p.key !== cloudConfig.provider ? async () => {
                          try {
                            await setCloudProvider(p.key);
                            // Reload full config to get correct has_api_key for new provider
                            const updated = await getCloudConfig();
                            setCloudConfig(updated);
                            setApiKeyInput("");
                            setApiKeySaved(false);
                            onToast(`Switched to ${p.name}`, "success");
                          } catch (err: any) {
                            onToast(err.toString(), "error");
                          }
                        } : undefined}
                      >
                        <div className="model-option-header">
                          <span className={`model-option-dot ${p.key === cloudConfig.provider ? "active" : ""}`} />
                          <span className="model-option-name">{p.name}</span>
                        </div>
                        <span className="model-option-desc">{p.description}</span>
                      </div>
                    ))}
                  </div>

                  {/* API key input */}
                  <div style={{ marginTop: 8 }}>
                    <div className="si-row" style={{ flexDirection: "column", alignItems: "stretch", gap: 6 }}>
                      <span className="si-label">API Key</span>
                      <div style={{ display: "flex", gap: 6 }}>
                        <input
                          type="password"
                          placeholder={cloudConfig.has_api_key ? "••••••••  (key saved)" : `Enter ${cloudConfig.provider_name} API key`}
                          value={apiKeyInput}
                          onChange={(e) => { setApiKeyInput(e.target.value); setApiKeySaved(false); }}
                          style={{
                            flex: 1,
                            padding: "6px 10px",
                            borderRadius: "var(--radius-sm)",
                            border: "1px solid var(--border)",
                            background: "var(--surface)",
                            color: "var(--text-primary)",
                            fontSize: "0.8rem",
                            fontFamily: "monospace",
                          }}
                        />
                        <button
                          className="sa-btn sa-btn-accent"
                          disabled={!apiKeyInput.trim() || apiKeySaved}
                          onClick={async () => {
                            try {
                              await setCloudApiKey(apiKeyInput.trim());
                              setCloudConfig({ ...cloudConfig, has_api_key: true });
                              setApiKeySaved(true);
                              onToast("API key saved", "success");
                            } catch (err: any) {
                              onToast(err.toString(), "error");
                            }
                          }}
                          style={{ whiteSpace: "nowrap" }}
                        >
                          {apiKeySaved ? "Saved" : "Save"}
                        </button>
                      </div>
                    </div>
                  </div>

                  {/* Session usage stats */}
                  {cloudUsage && cloudUsage.total_images > 0 && (
                    <div className="settings-info" style={{ marginTop: 10 }}>
                      <div className="si-row">
                        <span className="si-label">Session usage</span>
                        <span className="si-value">
                          {cloudUsage.total_images} image{cloudUsage.total_images !== 1 ? "s" : ""} — est. ${cloudUsage.total_estimated_cost.toFixed(4)}
                        </span>
                      </div>
                      {cloudUsage.by_provider.map((p) => (
                        <div className="si-row" key={p.provider}>
                          <span className="si-label" style={{ fontSize: "0.7rem", color: "var(--text-tertiary)" }}>
                            {p.provider_name}
                          </span>
                          <span className="si-value" style={{ fontSize: "0.7rem", color: "var(--text-tertiary)" }}>
                            {p.image_count} img — ${p.estimated_cost.toFixed(4)}
                          </span>
                        </div>
                      ))}
                      <div style={{ marginTop: 6 }}>
                        <button
                          className="sa-btn"
                          style={{ fontSize: "0.7rem", padding: "3px 8px" }}
                          onClick={async () => {
                            await resetCloudUsage();
                            setCloudUsage({ total_images: 0, total_estimated_cost: 0, by_provider: [] });
                            onToast("Usage counter reset", "info");
                          }}
                        >
                          Reset counter
                        </button>
                      </div>
                    </div>
                  )}
                </>
              )}
            </>
          )}
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
