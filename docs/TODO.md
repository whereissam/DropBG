# DropBG — TODO

## Phase 1: Core Engine (Rust Backend)

- [ ] Set up Tauri project scaffold
- [ ] Add `ort` (ONNX Runtime) dependency with CoreML support
- [ ] Add `image` crate for PNG read/write
- [ ] Download and integrate BiRefNet `.onnx` model
- [ ] Implement single-image background removal pipeline:
  - [ ] Load image → preprocess (resize to model input)
  - [ ] Run inference → get mask
  - [ ] Upscale mask to original resolution
  - [ ] Apply mask as alpha channel → save transparent PNG
- [ ] Implement edge refinement (smoothing, small-hole filling)
- [ ] Implement auto-crop of transparent edges

## Phase 2: UI (Tauri Frontend)

- [ ] Design minimal drag-and-drop interface
- [ ] Show processing spinner / progress indicator
- [ ] Display before/after preview
- [ ] Add "Save" / "Save As" buttons
- [ ] Support drag-and-drop of multiple files (batch mode)
- [ ] Show batch progress (x / n completed)

## Phase 3: Batch Processing

- [ ] Queue system for multiple images
- [ ] Auto-name output files with `_nobg.png` suffix
- [ ] Option to save outputs to original folder or custom folder
- [ ] Parallel inference (respect memory limits)

## Phase 4: Background Replacement

- [ ] Solid color replacement (white, black, custom picker)
- [ ] Custom image as new background
- [ ] Preset backgrounds (gradient, radial lines for YouTube thumbnails)

## Phase 5: Polish & Ship

- [ ] App icon and branding
- [ ] macOS code signing & notarization
- [ ] DMG installer build
- [ ] Landing page / GitHub releases
- [ ] Performance benchmarks on Apple Silicon (M1/M2/M3)

## Stretch Goals

- [ ] Figma plugin companion (thin plugin → DropBG local API)
- [ ] Portrait-specific model routing (MODNet for faces)
- [ ] Video background removal (frame-by-frame)
- [ ] Windows / Linux support
