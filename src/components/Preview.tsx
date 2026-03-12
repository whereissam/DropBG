import { useState, useEffect } from "react";

interface Props {
  originalUrl: string | null;
  resultBase64: string;
}

export default function Preview({ originalUrl, resultBase64 }: Props) {
  const [showOriginal, setShowOriginal] = useState(false);

  useEffect(() => {
    function handleKey(e: KeyboardEvent) {
      if (e.key === " " || e.code === "Space") {
        e.preventDefault();
        setShowOriginal((v) => !v);
      }
    }
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, []);

  const src = showOriginal && originalUrl
    ? originalUrl
    : `data:image/png;base64,${resultBase64}`;

  return (
    <div className="preview-container">
      <div className="canvas-area">
        <div className="checkerboard">
          <img src={src} alt={showOriginal ? "Original" : "Result"} className="preview-img" />
        </div>
      </div>
      <div className="hint-bar">
        <span
          className={`toggle ${showOriginal ? "active" : ""}`}
          onClick={() => setShowOriginal((v) => !v)}
          role="button"
          tabIndex={0}
          onKeyDown={(e) => e.key === "Enter" && setShowOriginal((v) => !v)}
        >
          {showOriginal ? "Showing Original" : "Showing Result"}
        </span>
        <span className="shortcut">Press Space to toggle</span>
      </div>
    </div>
  );
}
