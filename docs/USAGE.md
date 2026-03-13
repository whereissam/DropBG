# DropBG — Usage Guide

## Installation

### From Source (Development)

> Prerequisites: [Rust toolchain](https://rustup.rs/), [Bun](https://bun.sh/)

```bash
git clone https://github.com/whereissam/DropBG && cd DropBG
bun install
cargo tauri dev
```

### From DMG (Release)

1. Download the latest `.dmg` from [GitHub Releases](https://github.com/whereissam/DropBG/releases)
2. Open the DMG and drag **DropBG** to your Applications folder
3. On first launch, macOS will block the app because it's unsigned:

#### Enabling an Unsigned App on macOS

Since DropBG is distributed without an Apple Developer certificate, macOS Gatekeeper will block it. Here's how to allow it:

**Method 1 — System Settings (recommended)**

1. Try to open DropBG — you'll see "DropBG can't be opened because Apple cannot check it for malicious software"
2. Open **System Settings → Privacy & Security**
3. Scroll down — you'll see a message: *"DropBG" was blocked from use because it is not from an identified developer*
4. Click **Open Anyway**
5. Enter your password, then click **Open** in the confirmation dialog

**Method 2 — Terminal**

```bash
# Remove the quarantine attribute
xattr -cr /Applications/DropBG.app

# Then open normally
open /Applications/DropBG.app
```

**Method 3 — Right-click**

1. Right-click (or Control-click) the app in Finder
2. Select **Open** from the context menu
3. Click **Open** in the dialog

> After allowing once, macOS remembers your choice and won't block it again.

---

## First Launch — Model Setup

On first launch, DropBG asks you to download an AI model. No model is bundled with the app to keep the download small.

1. Choose your model (default: **BiRefNet Lite**, ~200 MB)
2. Click **Download** — progress is shown in real time
3. The model is saved to `~/Downloads/DropBG/` by default (configurable in Settings)
4. Once downloaded, the model works 100% offline

## Removing Backgrounds

### Single Image

1. **Drag and drop** an image onto the app, or click to browse
2. Wait for processing (progress steps shown: loading model → preprocessing → AI inference → applying mask)
3. **Preview** the result — press **Space** to toggle between original and transparent
4. Click **Save PNG** to export

### Batch Processing

1. **Drag and drop multiple images** at once (or select multiple in the file picker)
2. All images are processed sequentially with per-image progress
3. Output files are saved automatically as `{name}_nobg.png` in your configured save folder
4. Click **Open Folder** when done to reveal results in Finder

## Available AI Models

DropBG supports multiple background removal models. Switch between them in **Settings → AI Model**.

| Model | Size | Best For | License |
|-------|------|----------|---------|
| **BiRefNet Lite** | ~200 MB | Fast general use, most images | MIT |
| **BiRefNet Full** | ~900 MB | Complex backgrounds, high detail | MIT |
| **BEN2** | ~219 MB | Hair, fine edges, complex scenes | MIT |
| **RMBG 2.0** | ~514 MB | Best overall quality | CC BY-NC 4.0 (manual download) |
| **MODNet** | ~13 MB | Portraits and people (lightweight) | Apache 2.0 |

- Models with auto-download: click to switch, then download if needed
- **RMBG 2.0** requires manual download from [HuggingFace](https://huggingface.co/briaai/RMBG-2.0/blob/main/onnx/model_fp16.onnx) (gated model — accept terms, download, rename to `rmbg2_fp16.onnx`, place in model folder)
- You can download multiple models and switch between them without restarting

## Post-Processing Tools

### Auto-Crop

Click **Auto-Crop** in the toolbar to trim transparent edges from the result. Adds 4px padding by default.

### Background Replacement

After removing the background, use the **Background** panel at the bottom:

- **Solid** — 6 preset colors + custom color picker
- **Gradient** — 6 preset gradients (sunset, ocean, purple, dark, mint, fire)
- **Image** — pick any image file as the new background
- **Transparent** — click the checkerboard swatch to reset to transparent

### AI Upscale

Click **Upscale** in the toolbar to enhance resolution using Real-ESRGAN:

- **2x** — doubles the resolution
- **4x** — quadruples the resolution (native model output)
- Requires the upscale model (~64 MB) — download it in **Settings → AI Upscale**

## Settings

Open Settings via the gear icon (top-right corner).

### AI Model

- View active model, status, and file location
- Switch between all available models
- Download or delete models
- Change model storage location

### AI Upscale

- Download/manage the Real-ESRGAN upscale model (~64 MB)

### Save Location

- Set the default folder for saved images and batch output
- The save dialog opens here by default
- Auto-updates when you pick a different folder while saving

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| **Space** | Toggle between original and transparent preview |

## File Support

- **Input**: JPEG, PNG, WebP, BMP, TIFF, GIF
- **Output**: PNG (with alpha channel)

## Performance

- Inference runs via ONNX Runtime with CoreML acceleration on Apple Silicon
- Typical processing time: 2-5 seconds per image (M1/M2/M3)
- The Neural Engine handles inference — CPU stays free for other tasks
- First inference after launch is slower (model loading), subsequent images are faster

## Troubleshooting

### "Model not downloaded"

Go to Settings and download the model, or check that the model file exists in the configured model directory.

### Processing fails on certain images

Try switching to a different model in Settings. **BiRefNet Full** or **BEN2** handle complex backgrounds better than the Lite model.

### App won't open on macOS

See [Enabling an Unsigned App on macOS](#enabling-an-unsigned-app-on-macos) above.

### High memory usage

Large images (8000x6000+) during upscaling can use significant memory. Close other apps if needed, or use 2x instead of 4x upscale.
