import { useState, useEffect, useCallback } from "react";
import { checkModelReady, isOnboardingDone, completeOnboarding, downloadModel, openPathInFinder, getModelInfo, getOutputDir, removeBackgroundBatch, appleVisionAvailable, removeBackgroundAppleVision, getCloudConfig, removeBackgroundCloud } from "./tauri";
import DropZone from "./components/DropZone";
import Onboarding from "./components/Onboarding";
import ModelSetup from "./components/ModelSetup";
import Preview from "./components/Preview";
import Toolbar from "./components/Toolbar";
import Settings from "./components/Settings";
import Toast from "./components/Toast";
import BatchList, { type BatchItem } from "./components/BatchList";
import BgReplace from "./components/BgReplace";
import "./App.css";

type Stage = "loading" | "onboarding" | "setup" | "downloading" | "ready";

interface ToastData {
  id: number;
  message: string;
  type: "success" | "error" | "info";
  action?: { label: string; onClick: () => void };
}

let toastId = 0;

export default function App() {
  const [stage, setStage] = useState<Stage>("loading");
  const [downloadProgress, setDownloadProgress] = useState(0);
  const [downloadError, setDownloadError] = useState<string | null>(null);
  const [toasts, setToasts] = useState<ToastData[]>([]);
  const [showSettings, setShowSettings] = useState(false);
  const [useAppleVision, setUseAppleVision] = useState(false);

  // Single image state
  const [originalPath, setOriginalPath] = useState<string | null>(null);
  const [originalUrl, setOriginalUrl] = useState<string | null>(null);
  const [transparentBase64, setTransparentBase64] = useState<string | null>(null); // always the alpha result
  const [resultBase64, setResultBase64] = useState<string | null>(null); // may have bg applied
  const [processing, setProcessing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Batch state
  const [batchItems, setBatchItems] = useState<BatchItem[]>([]);
  const [batchRunning, setBatchRunning] = useState(false);
  const [batchOutputDir, setBatchOutputDir] = useState("");

  const addToast = useCallback((
    message: string,
    type: "success" | "error" | "info",
    action?: { label: string; onClick: () => void },
  ) => {
    const id = ++toastId;
    setToasts((prev) => [...prev, { id, message, type, action }]);
  }, []);

  const removeToast = useCallback((id: number) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  useEffect(() => {
    (async () => {
      try {
        const onboarded = await isOnboardingDone();
        if (!onboarded) {
          setStage("onboarding");
          return;
        }
        const ready = await checkModelReady();
        if (ready) {
          setStage("ready");
        } else {
          // Check if cloud mode is enabled — skip model setup
          const cloud = await getCloudConfig().catch(() => null);
          if (cloud?.enabled && cloud.has_api_key) {
            setStage("ready");
          } else {
            // No model downloaded — use Apple Vision as fallback if available
            const hasVision = await appleVisionAvailable().catch(() => false);
            if (hasVision) {
              setUseAppleVision(true);
              setStage("ready");
            } else {
              setStage("setup");
            }
          }
        }
      } catch {
        setStage("setup");
      }
    })();
  }, []);

  const startDownload = useCallback(async () => {
    setStage("downloading");
    setDownloadProgress(0);
    setDownloadError(null);

    try {
      const { listen } = await import("@tauri-apps/api/event");
      const unlisten = await listen<number>("download-progress", (e) => {
        setDownloadProgress(e.payload);
      });

      try {
        await downloadModel();
        setUseAppleVision(false);
        setStage("ready");

        const info = await getModelInfo().catch(() => null);
        addToast(
          "Model downloaded! Ready to remove backgrounds.",
          "success",
          info ? {
            label: "Show in Finder",
            onClick: () => openPathInFinder(info.model_path).catch(() => {}),
          } : undefined,
        );
      } catch (e: any) {
        setDownloadError(e.toString());
        setStage("setup");
        addToast("Download failed: " + e.toString(), "error");
      } finally {
        unlisten();
      }
    } catch (e: any) {
      setDownloadError(e.toString());
      setStage("setup");
    }
  }, [addToast]);

  function reset() {
    if (originalUrl) URL.revokeObjectURL(originalUrl);
    setOriginalPath(null);
    setOriginalUrl(null);
    setTransparentBase64(null);
    setResultBase64(null);
    setProcessing(false);
    setError(null);
    setBatchItems([]);
    setBatchRunning(false);
    setBatchOutputDir("");
  }

  async function handleProcessSingle(filePath: string) {
    setOriginalPath(filePath);
    setProcessing(true);
    setError(null);
    setTransparentBase64(null);
    setResultBase64(null);
    setOriginalUrl(null);

    try {
      try {
        const { readFile } = await import("@tauri-apps/plugin-fs");
        const bytes = await readFile(filePath);
        const blob = new Blob([bytes]);
        setOriginalUrl(URL.createObjectURL(blob));
      } catch { /* skip preview if fs read fails */ }

      let base64: string;
      // Check if cloud mode is enabled
      const cloud = await getCloudConfig().catch(() => null);
      if (cloud?.enabled && cloud.has_api_key) {
        base64 = await removeBackgroundCloud(filePath);
      } else if (useAppleVision) {
        base64 = await removeBackgroundAppleVision(filePath);
      } else {
        const { removeBackground } = await import("./tauri");
        base64 = await removeBackground(filePath);
      }
      setTransparentBase64(base64);
      setResultBase64(base64);
    } catch (e: any) {
      setError(e.toString());
    } finally {
      setProcessing(false);
    }
  }

  async function handleProcessBatch(filePaths: string[]) {
    const outputDir = await getOutputDir().catch(() => "");
    if (!outputDir) {
      addToast("No output directory configured. Set one in Settings.", "error");
      return;
    }

    setBatchOutputDir(outputDir);
    setBatchRunning(true);
    setBatchItems(
      filePaths.map((p, i) => ({
        index: i,
        filename: p.split("/").pop() ?? `image_${i}`,
        status: "pending" as const,
      })),
    );

    try {
      const { listen } = await import("@tauri-apps/api/event");
      const unlisten = await listen<{
        index: number;
        total: number;
        filename: string;
        status: string;
        error: string | null;
        output_path: string | null;
      }>("batch-progress", (e) => {
        const p = e.payload;
        setBatchItems((prev) =>
          prev.map((item) =>
            item.index === p.index
              ? {
                  ...item,
                  status: p.status as BatchItem["status"],
                  error: p.error ?? undefined,
                  outputPath: p.output_path ?? undefined,
                }
              : item,
          ),
        );
      });

      try {
        const results = await removeBackgroundBatch(filePaths, outputDir);
        const successCount = results.length;
        const failCount = filePaths.length - successCount;

        addToast(
          `Batch complete: ${successCount} processed${failCount > 0 ? `, ${failCount} failed` : ""}`,
          failCount > 0 ? "info" : "success",
          {
            label: "Open Folder",
            onClick: () => openPathInFinder(outputDir).catch(() => {}),
          },
        );
      } finally {
        unlisten();
      }
    } catch (e: any) {
      addToast("Batch processing failed: " + e.toString(), "error");
    } finally {
      setBatchRunning(false);
    }
  }

  const isBatchMode = batchItems.length > 0;

  return (
    <div className="app-container">
      {/* Toast stack */}
      <div className="toast-stack">
        {toasts.map((t) => (
          <Toast
            key={t.id}
            message={t.message}
            type={t.type}
            action={t.action}
            onClose={() => removeToast(t.id)}
          />
        ))}
      </div>

      {/* Settings panel */}
      {showSettings && (
        <Settings
          onClose={() => setShowSettings(false)}
          onModelDeleted={() => setStage("setup")}
          onToast={(msg, type) => addToast(msg, type)}
        />
      )}

      {/* Settings gear button */}
      {stage !== "loading" && (
        <button
          className="gear-btn"
          onClick={() => setShowSettings(true)}
          title="Settings"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <circle cx="12" cy="12" r="3" />
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
          </svg>
        </button>
      )}

      {stage === "loading" ? (
        <div className="center-content">
          <div className="spinner" />
        </div>
      ) : stage === "onboarding" ? (
        <Onboarding
          onComplete={async () => {
            await completeOnboarding().catch(() => {});
            const ready = await checkModelReady().catch(() => false);
            if (ready) {
              setStage("ready");
            } else {
              const hasVision = await appleVisionAvailable().catch(() => false);
              if (hasVision) {
                setUseAppleVision(true);
                setStage("ready");
              } else {
                setStage("setup");
              }
            }
          }}
        />
      ) : stage === "setup" || stage === "downloading" ? (
        <ModelSetup
          downloading={stage === "downloading"}
          progress={downloadProgress}
          error={downloadError}
          onDownload={startDownload}
          onSkipWithVision={() => {
            setUseAppleVision(true);
            setStage("ready");
            addToast("Using Apple Vision. Download a model in Settings for better quality.", "info");
          }}
        />
      ) : isBatchMode ? (
        <BatchList
          items={batchItems}
          total={batchItems.length}
          outputDir={batchOutputDir}
          onDone={reset}
        />
      ) : resultBase64 ? (
        <>
          <Toolbar
            originalPath={originalPath}
            resultBase64={resultBase64}
            onReset={reset}
            onUpdateResult={setResultBase64}
            onToast={addToast}
          />
          <Preview originalUrl={originalUrl} resultBase64={resultBase64} />
          {transparentBase64 && (
            <BgReplace
              transparentBase64={transparentBase64}
              onApply={setResultBase64}
            />
          )}
        </>
      ) : (
        <DropZone
          ready={stage === "ready"}
          processing={processing}
          error={error}
          onProcess={handleProcessSingle}
          onProcessBatch={handleProcessBatch}
          onReset={reset}
        />
      )}
    </div>
  );
}
