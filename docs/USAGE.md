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

## First Launch

DropBG works the moment you open it — no download required.

- **Default: Apple Vision** is built into macOS 14+ and runs instantly on the Neural Engine. Good quality for most photos, zero setup.
- **Optional: download a specialized AI model** if you want higher quality (complex hair, fur, fine edges, product shots). Models are 13 MB – 900 MB depending on which you pick.

To add an AI model:

1. Open **Settings → AI Model** and choose the model you want
2. Click **Download** — progress is shown in real time
3. Models are saved to `~/Downloads/DropBG/` by default (configurable in Settings)
4. Once downloaded, the model works 100% offline; you can switch models without restarting

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

DropBG ships with **Apple Vision** as the zero-download default. You can add specialized AI models in **Settings → AI Model** for higher quality.

### Curated lineup

| Model | Tier | Size | Best For | License |
|-------|------|------|----------|---------|
| **Apple Vision** | Default | Built-in | Zero-setup; good quality on macOS 14+ | Apple system framework |
| **BiRefNet Lite** | Recommended | ~200 MB | Fast local background removal | MIT |
| **BiRefNet General** | Quality | ~490 MB | Higher-quality general cutouts | MIT |
| **BiRefNet Matting** | Best edges | Manual | Alpha mattes for hair, fur, transparency | MIT |
| **BiRefNet HR-matting** | High-resolution | Manual | Alpha mattes at 2048×2048 — best for large product / portrait shots | MIT |
| **BiRefNet Dynamic** | Native resolution | Manual | Arbitrary image sizes (256–2304 px) | MIT |
| **BiRefNet Dynamic Matting** | Native-res alpha | Manual | Soft alpha mattes at the image's own size — hair/fur/glass without a forced square resize | MIT |
| **BEN2** | Edge detail | ~219 MB | Experimental alternative for difficult boundaries — benchmark against BiRefNet Matting first | MIT |
| **RMBG 2.0** | Product | ~514 MB | Ecommerce / product shots | **CC BY-NC 4.0** (non-commercial only) |
| **MODNet** | Lightweight | ~13 MB | Portraits / legacy use | Apache 2.0 |

### Also shipped (advanced)

- **BiRefNet Full** (~900 MB) — superseded by BiRefNet General for most use cases
- **BiRefNet Portrait** (~490 MB) — specialized portrait model (auto-selected when faces detected, if Auto routing is on)
- **InSPyReNet** (~300 MB, manual) — strong on fuzzy edges and hair strands

### Notes

- **Auto-download**: most models can be downloaded directly from Settings; download progress is shown in the model picker.
- **Manual download** is required for RMBG 2.0, BiRefNet Matting, BiRefNet HR-matting, BiRefNet Dynamic, and BiRefNet Dynamic Matting. The BiRefNet matting/dynamic variants have no pre-built ONNX — export them with the `scripts/export_*_onnx.py` script named in Settings, then copy the `.onnx` into your model folder. RMBG 2.0 is a gated download: accept the terms on its HuggingFace page (linked in Settings), download `model_fp16.onnx`, and rename per the prompt.
- **BiRefNet HR-matting** is trained at 2048×2048 and uses ~4× more memory than the 1024-input models. Close other apps before running heavy batches on machines with under 16 GB RAM.
- You can download multiple models and switch between them without restarting.
- **Two-stage edge refinement**: when enabled, a coarse BiRefNet mask is refined by ViTMatte Small (~28 MB) for cleaner hair and fur boundaries.

### Model license callout

DropBG itself is MIT-licensed. **Model weights have their own licenses, and not all of them are commercial.**

- **RMBG 2.0** is released under **CC BY-NC 4.0** — non-commercial use only. Commercial use requires an agreement with [BRIA](https://bria.ai). For a commercial-safe RMBG path, the **fal.ai BRIA endpoint** is on the cloud roadmap (see below).
- **MODNet** is Apache 2.0 — generally commercial-safe.
- **BiRefNet family and BEN2** are MIT — generally commercial-safe; verify each upstream repo before shipping outputs.

If you use DropBG for client work or ecommerce, check the license of the model you select before exporting results.

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

## Cloud APIs (optional)

Local processing is the default. If you want to offload to a GPU — for large batches, the very highest-quality endpoints, or to avoid downloading model weights — DropBG supports a few cloud providers. **Cloud mode is off by default and your images are only sent when you explicitly enable it.**

### Setup

1. Open **Settings → Cloud**
2. Pick a provider and paste your API key — keys are stored in DropBG's config file on your Mac, never transmitted anywhere except the provider you chose
3. Toggle **Use cloud** to switch between local and cloud inference at any time
4. Per-session usage and estimated cost are shown in Settings

### Shipped today

| Provider | Endpoint | Notes |
|----------|----------|-------|
| **Replicate** | Community BiRefNet model | Pay-per-run or compute-time pricing — check the model page |
| **fal.ai BiRefNet** | `fal-ai/birefnet` | Fast and reliable |
| **fal.ai BRIA RMBG 2.0** | `fal-ai/bria/background/remove` | Commercial-safe RMBG via API — avoids the local non-commercial license issue |
| **fal.ai Ideogram Remove Background** | `fal-ai/ideogram/remove-background` | High-quality cutouts with clean edges |
| **remove.bg** | Proprietary | Mature API, paid credits; useful as a quality benchmark |
| **Photoroom** | `sdk.photoroom.com/v1/segment` | Strong product-photo workflow for ecommerce users |

When **fal.ai** is selected, pick the endpoint in **Settings → Cloud → Endpoint**. The Endpoint sub-picker only appears when fal.ai is active.

See [TODO.md](TODO.md) Phase 10 for what landed.

### Pricing notes

Cloud pricing changes frequently. DropBG doesn't hardcode prices in the UI — open the provider's pricing page directly. As a rough guide at the time of writing:

- **Replicate** bills by compute time or per-run depending on the model
- **fal.ai BRIA RMBG 2.0** is listed at ~$0.018 per generation (provider page, 2026-05)
- **fal.ai Ideogram Remove Background** — check the provider page for current pricing
- **remove.bg** uses a credit system; the first 50 API calls per month are free
- **Photoroom** uses monthly subscription credits based on images processed — see the provider's pricing docs

Always check the provider page for current pricing before running large batches.

## Settings

Open Settings via the gear icon (top-right corner).

### AI Model

- Default: **Apple Vision** (built into macOS 14+, no download needed)
- Switch to any downloaded AI model from the picker
- Download / delete model weights
- Change model storage location
- Optional: enable **Auto routing** to let DropBG pick BiRefNet Portrait automatically when faces are detected

### AI Upscale

- Download / manage the Real-ESRGAN upscale model (~64 MB)

### Cloud

- Add API keys for Replicate, fal.ai, remove.bg, or Photoroom (each stored locally; never shared)
- When fal.ai is selected, pick the endpoint sub-variant (BiRefNet / BRIA RMBG 2.0 / Ideogram)
- Toggle cloud mode on/off
- View per-session usage and estimated cost

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

Go to Settings and download the model, or check that the model file exists in the configured model directory. If you don't want to download anything, switch back to **Apple Vision** — it's built into macOS and always available on macOS 14+.

### Apple Vision result looks worse than expected

Apple Vision is a fast, general-purpose segmenter. For tricky images (fine hair, fur, motion blur, complex backgrounds), switch to **BiRefNet General**, **BiRefNet Matting**, or **BEN2** in Settings. For product photos, **RMBG 2.0** is strongest (but is non-commercial — see the license callout above).

### Processing fails on certain images

Try switching to a different model in Settings. **BiRefNet General** or **BEN2** handle complex backgrounds better than the Lite model. If a local model keeps failing, you can also temporarily switch to a cloud provider in **Settings → Cloud**.

### App won't open on macOS

See [Enabling an Unsigned App on macOS](#enabling-an-unsigned-app-on-macos) above.

### High memory usage

Large images (8000x6000+) during upscaling can use significant memory. Close other apps if needed, or use 2x instead of 4x upscale.
