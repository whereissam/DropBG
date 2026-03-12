import { useState } from "react";
import {
  replaceBackgroundColor,
  replaceBackgroundGradient,
  replaceBackgroundImage,
} from "../tauri";

interface Props {
  /** The transparent (alpha) result base64 — always use this as source */
  transparentBase64: string;
  onApply: (newBase64: string) => void;
}

const SOLID_PRESETS = [
  { label: "White", color: "#ffffff", r: 255, g: 255, b: 255 },
  { label: "Black", color: "#000000", r: 0, g: 0, b: 0 },
  { label: "Red", color: "#e94560", r: 233, g: 69, b: 96 },
  { label: "Blue", color: "#2563eb", r: 37, g: 99, b: 235 },
  { label: "Green", color: "#22c55e", r: 34, g: 197, b: 94 },
  { label: "Gray", color: "#6b7280", r: 107, g: 114, b: 128 },
];

const GRADIENT_PRESETS = [
  { label: "Sunset", from: [255, 99, 71], to: [255, 195, 0] },
  { label: "Ocean", from: [0, 119, 182], to: [0, 180, 216] },
  { label: "Purple", from: [139, 92, 246], to: [236, 72, 153] },
  { label: "Dark", from: [30, 30, 30], to: [60, 60, 80] },
  { label: "Mint", from: [134, 239, 172], to: [59, 130, 246] },
  { label: "Fire", from: [220, 38, 38], to: [251, 146, 60] },
];

type Tab = "solid" | "gradient" | "image";

export default function BgReplace({ transparentBase64, onApply }: Props) {
  const [tab, setTab] = useState<Tab>("solid");
  const [customColor, setCustomColor] = useState("#ffffff");
  const [loading, setLoading] = useState(false);

  async function handleSolid(r: number, g: number, b: number) {
    setLoading(true);
    try {
      const result = await replaceBackgroundColor(transparentBase64, r, g, b);
      onApply(result);
    } catch (e) {
      console.error("BG replace error:", e);
    } finally {
      setLoading(false);
    }
  }

  async function handleGradient(from: number[], to: number[]) {
    setLoading(true);
    try {
      const result = await replaceBackgroundGradient(
        transparentBase64,
        from[0], from[1], from[2],
        to[0], to[1], to[2],
      );
      onApply(result);
    } catch (e) {
      console.error("BG gradient error:", e);
    } finally {
      setLoading(false);
    }
  }

  async function handleCustomColor() {
    const r = parseInt(customColor.slice(1, 3), 16);
    const g = parseInt(customColor.slice(3, 5), 16);
    const b = parseInt(customColor.slice(5, 7), 16);
    await handleSolid(r, g, b);
  }

  async function handleImagePick() {
    setLoading(true);
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        multiple: false,
        filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg", "webp"] }],
      });
      if (selected) {
        const result = await replaceBackgroundImage(transparentBase64, selected as string);
        onApply(result);
      }
    } catch (e) {
      console.error("BG image error:", e);
    } finally {
      setLoading(false);
    }
  }

  async function handleRemoveBg() {
    // Reset to transparent — just restore the original
    onApply(transparentBase64);
  }

  return (
    <div className="bg-replace">
      <div className="bg-tabs">
        <button className={`bg-tab ${tab === "solid" ? "active" : ""}`} onClick={() => setTab("solid")}>Solid</button>
        <button className={`bg-tab ${tab === "gradient" ? "active" : ""}`} onClick={() => setTab("gradient")}>Gradient</button>
        <button className={`bg-tab ${tab === "image" ? "active" : ""}`} onClick={() => setTab("image")}>Image</button>
      </div>

      {loading && <div className="bg-loading">Applying...</div>}

      {tab === "solid" && !loading && (
        <div className="bg-presets">
          <button
            className="bg-swatch bg-swatch-transparent"
            onClick={handleRemoveBg}
            title="Transparent (no background)"
          />
          {SOLID_PRESETS.map((p) => (
            <button
              key={p.label}
              className="bg-swatch"
              style={{ background: p.color }}
              onClick={() => handleSolid(p.r, p.g, p.b)}
              title={p.label}
            />
          ))}
          <div className="bg-custom-color">
            <input
              type="color"
              value={customColor}
              onChange={(e) => setCustomColor(e.target.value)}
              className="bg-color-input"
            />
            <button className="bg-apply-custom" onClick={handleCustomColor}>
              Apply
            </button>
          </div>
        </div>
      )}

      {tab === "gradient" && !loading && (
        <div className="bg-presets">
          <button
            className="bg-swatch bg-swatch-transparent"
            onClick={handleRemoveBg}
            title="Transparent (no background)"
          />
          {GRADIENT_PRESETS.map((p) => (
            <button
              key={p.label}
              className="bg-swatch"
              style={{
                background: `linear-gradient(180deg, rgb(${p.from.join(",")}) 0%, rgb(${p.to.join(",")}) 100%)`,
              }}
              onClick={() => handleGradient(p.from, p.to)}
              title={p.label}
            />
          ))}
        </div>
      )}

      {tab === "image" && !loading && (
        <div className="bg-image-pick">
          <button
            className="bg-swatch bg-swatch-transparent"
            onClick={handleRemoveBg}
            title="Transparent (no background)"
          />
          <button className="bg-pick-btn" onClick={handleImagePick}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
              <circle cx="8.5" cy="8.5" r="1.5" />
              <polyline points="21 15 16 10 5 21" />
            </svg>
            Choose Background Image
          </button>
        </div>
      )}
    </div>
  );
}
