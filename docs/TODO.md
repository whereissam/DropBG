# DropBG — TODO

## Phase 1: Core Engine (Rust Backend) ✅

- [x] Set up Tauri 2 project scaffold
- [x] Add `ort` (ONNX Runtime) dependency with CoreML support
- [x] Add `image` crate for PNG read/write
- [x] Download and integrate BiRefNet Lite fp16 ONNX model
- [x] Implement single-image background removal pipeline:
  - [x] Load image → preprocess (resize to 1024×1024, normalize)
  - [x] Run inference → get mask
  - [x] Upscale mask to original resolution
  - [x] Apply mask as alpha channel → base64 PNG
- [x] Implement edge refinement (smoothing, small-hole filling)
- [x] Implement auto-crop of transparent edges

## Phase 2: UI (Tauri Frontend) ✅

- [x] Design minimal drag-and-drop interface
- [x] Show processing spinner / progress indicator with step labels
- [x] Display before/after preview (Space key toggle)
- [x] Add "Save PNG" button with save dialog
- [x] Model download consent screen (not auto-download)
- [x] Configurable model download location
- [x] Configurable default save/output location
- [x] Settings panel (model info, delete, relocate, output dir)
- [x] Toast notifications with actions
- [x] App logo and branding
- [x] Support drag-and-drop of multiple files (batch mode)
- [x] Show batch progress (x / n completed)

## Phase 3: Batch Processing ✅

- [x] Queue system for multiple images
- [x] Auto-name output files with `_nobg.png` suffix
- [x] Save outputs to configurable output folder
- [x] Batch progress UI (list view with per-image status)
- [x] Sequential batch (CoreML already saturates Neural Engine — parallel would be slower)

## Phase 4: Model Quality ✅

- [x] Add full BiRefNet model (~900 MB) as "High Quality" option for complex backgrounds
- [x] Model selector in Settings (multi-model: BiRefNet Lite, BiRefNet Full, BEN2, MODNet)
- [x] Auto-download selected model variant
- [x] Allow switching models without restarting (hot-swap via session clear)
- [x] BEN2 (219 MB) — best on hair & fine edges, MIT license
- [x] MODNet (13 MB) — lightweight, optimized for portraits & people

## Phase 5: Background Replacement ✅

- [x] Solid color replacement (white, black, red, blue, green, gray + custom color picker)
- [x] Custom image as new background
- [x] Preset gradient backgrounds (sunset, ocean, purple, dark, mint, fire)
- [x] Transparent reset option (checkerboard swatch)

## Phase 6: Image Enhancement ✅

- [x] AI upscaling / super-resolution (Real-ESRGAN x4plus ONNX model)
- [x] Enhance image resolution after background removal
- [x] Configurable scale factor (2x, 4x)
- [x] Tile-based processing for large images (256px tiles with 16px padding)
- [x] Upscale model download/management in Settings

## Phase 7: Polish & Ship

- [x] App icon and branding
- [ ] macOS code signing & notarization
- [ ] DMG installer build
- [ ] Landing page / GitHub releases
- [ ] Performance benchmarks on Apple Silicon (M1/M2/M3)

## Stretch Goals

- [ ] Figma plugin companion (thin plugin → DropBG local API)
- [ ] Portrait-specific model routing (MODNet for faces)
- [ ] Video background removal (frame-by-frame)
- [ ] Windows / Linux support
