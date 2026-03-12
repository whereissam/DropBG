import { useEffect, useState } from "react";
import { openPathInFinder } from "../tauri";

export interface BatchItem {
  index: number;
  filename: string;
  status: "pending" | "processing" | "done" | "error";
  error?: string;
  outputPath?: string;
}

interface Props {
  items: BatchItem[];
  total: number;
  onDone: () => void;
  outputDir: string;
}

export default function BatchList({ items, total, onDone, outputDir }: Props) {
  const doneCount = items.filter((i) => i.status === "done").length;
  const errorCount = items.filter((i) => i.status === "error").length;
  const finished = doneCount + errorCount === total && total > 0;
  const percent = total > 0 ? ((doneCount + errorCount) / total) * 100 : 0;

  return (
    <div className="batch-container">
      <div className="batch-header">
        <h2>Batch Processing</h2>
        <span className="batch-count">
          {doneCount + errorCount} / {total}
        </span>
      </div>

      <div className="batch-progress-bar">
        <div className="progress-bar wide">
          <div className="progress-fill" style={{ width: `${percent}%` }} />
        </div>
      </div>

      <div className="batch-list">
        {items.map((item) => (
          <div key={item.index} className={`batch-item batch-${item.status}`}>
            <span className="batch-status-icon">
              {item.status === "pending" && "○"}
              {item.status === "processing" && <span className="batch-spinner" />}
              {item.status === "done" && "✓"}
              {item.status === "error" && "✗"}
            </span>
            <span className="batch-filename">{item.filename}</span>
            {item.status === "error" && (
              <span className="batch-error" title={item.error}>{item.error}</span>
            )}
            {item.status === "done" && item.outputPath && (
              <button
                className="batch-reveal"
                onClick={() => openPathInFinder(item.outputPath!).catch(() => {})}
                title="Show in Finder"
              >
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
                  <polyline points="15 3 21 3 21 9" />
                  <line x1="10" y1="14" x2="21" y2="3" />
                </svg>
              </button>
            )}
          </div>
        ))}
      </div>

      {finished && (
        <div className="batch-footer">
          <button
            className="btn btn-secondary"
            onClick={() => openPathInFinder(outputDir).catch(() => {})}
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
            </svg>
            Open Output Folder
          </button>
          <button className="btn btn-primary" onClick={onDone}>
            Done
          </button>
        </div>
      )}
    </div>
  );
}
