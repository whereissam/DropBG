import { useState, useEffect, useCallback } from "react";
import { checkModelReady, downloadModel, openPathInFinder, getModelInfo } from "./tauri";
import DropZone from "./components/DropZone";
import ModelSetup from "./components/ModelSetup";
import Preview from "./components/Preview";
import Toolbar from "./components/Toolbar";
import Settings from "./components/Settings";
import Toast from "./components/Toast";
import "./App.css";

type Stage = "loading" | "setup" | "downloading" | "ready";

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

  const [originalPath, setOriginalPath] = useState<string | null>(null);
  const [originalUrl, setOriginalUrl] = useState<string | null>(null);
  const [resultBase64, setResultBase64] = useState<string | null>(null);
  const [processing, setProcessing] = useState(false);
  const [error, setError] = useState<string | null>(null);

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
        const ready = await checkModelReady();
        setStage(ready ? "ready" : "setup");
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
        setStage("ready");

        // Get model info for the toast action
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
    setResultBase64(null);
    setProcessing(false);
    setError(null);
  }

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
      ) : stage === "setup" || stage === "downloading" ? (
        <ModelSetup
          downloading={stage === "downloading"}
          progress={downloadProgress}
          error={downloadError}
          onDownload={startDownload}
        />
      ) : resultBase64 ? (
        <>
          <Toolbar
            originalPath={originalPath}
            resultBase64={resultBase64}
            onReset={reset}
          />
          <Preview originalUrl={originalUrl} resultBase64={resultBase64} />
        </>
      ) : (
        <DropZone
          ready={stage === "ready"}
          processing={processing}
          error={error}
          onProcess={async (filePath) => {
            setOriginalPath(filePath);
            setProcessing(true);
            setError(null);
            setResultBase64(null);
            setOriginalUrl(null);

            try {
              try {
                const { readFile } = await import("@tauri-apps/plugin-fs");
                const bytes = await readFile(filePath);
                const blob = new Blob([bytes]);
                setOriginalUrl(URL.createObjectURL(blob));
              } catch { /* skip preview if fs read fails */ }

              const { removeBackground } = await import("./tauri");
              const base64 = await removeBackground(filePath);
              setResultBase64(base64);
            } catch (e: any) {
              setError(e.toString());
            } finally {
              setProcessing(false);
            }
          }}
          onReset={reset}
        />
      )}
    </div>
  );
}
