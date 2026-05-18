<p align="center">
  <img src="src/assets/logo.png" width="128" height="128" alt="DropBG Logo" />
</p>

<h1 align="center">DropBG</h1>

<p align="center">Local-first background remover for macOS.<br>No uploads by default. No subscriptions. No app-level resolution limits.</p>

<p align="center">
  <img src="src/assets/screenshot.png" width="600" alt="DropBG Screenshot" />
</p>

## What is DropBG?

DropBG is a local-first background remover for macOS.

Unlike remove.bg-style tools, your images stay on your Mac by default. Download one AI model once, then remove backgrounds offline with no subscriptions, no per-image fees, and no app-level resolution limits.

A cloud option is available if you want to bring your own API key — but it is opt-in, never the default.

## Why use DropBG instead of online tools?

| Feature             | DropBG                    | remove.bg / cloud tools     |
| ------------------- | ------------------------- | --------------------------- |
| Runs offline        | ✅ Yes                    | ❌ No                       |
| Images stay local   | ✅ By default             | ❌ Uploaded to server       |
| Per-image cost      | ✅ Free after setup       | ❌ Usually paid             |
| Resolution limits   | ✅ No app-level limit     | ⚠️ Often limited by plan    |
| Batch processing    | ✅ Yes                    | ⚠️ Depends on plan          |
| Cloud option        | ✅ Optional BYO key       | ✅ Built-in                 |

## Features

- **Drag-and-drop** single or multiple images
- **Multiple AI models** — choose faster, higher-quality, or lightweight models depending on your image
- **Cloud API support** — Replicate, fal.ai, remove.bg (bring your own key, opt-in)
- **Batch processing** with per-image progress and auto-naming
- **Background replacement** — solid colors, gradients, or custom images
- **AI upscaling** — 2x/4x super-resolution via Real-ESRGAN
- **Auto-crop** — trim transparent edges automatically
- **Before/after preview** — press Space to toggle
- **Configurable** — model location, save folder, model switching without restart

## Models

### Local models (run 100% offline)

| Model                | Tier              | Size     | Best For                                | License                                  |
| -------------------- | ----------------- | -------- | --------------------------------------- | ---------------------------------------- |
| **Apple Vision**     | Default           | Built-in | Zero-setup, good quality on macOS 14+   | Apple system framework                   |
| **BiRefNet Lite**    | Recommended       | ~200 MB  | Fast local background removal           | MIT                                      |
| **BiRefNet General** | Quality           | ~490 MB  | Higher-quality general cutouts          | MIT                                      |
| **BiRefNet Matting** | Best edges        | Manual   | Alpha mattes for hair, fur, transparency| MIT                                      |
| **BiRefNet HR-matting** | High-resolution | Manual   | Alpha mattes at 2048×2048 — large product / portrait shots | MIT                       |
| **BiRefNet Dynamic** | Native resolution | Manual   | Arbitrary image sizes (256–2304 px)     | MIT                                      |
| **BEN2**             | Edge detail       | ~219 MB  | Hair, fur, difficult boundaries (experimental) | MIT                              |
| **RMBG 2.0**         | Product           | ~514 MB  | Ecommerce / product shots               | **CC BY-NC 4.0** — non-commercial only \*|
| **MODNet**           | Lightweight       | ~13 MB   | Portraits, legacy use                   | Apache 2.0                               |

Also shipped: BiRefNet Portrait (specialized portrait) and InSPyReNet (advanced; fuzzy edges). These are available in Settings but not the curated lineup — pick BiRefNet General or Matting first.

> **\* Model licenses vary.** DropBG itself is MIT-licensed, but model weights have their own licenses. **RMBG 2.0 is released under CC BY-NC 4.0** — commercial use requires an agreement with BRIA. For a commercial-safe RMBG path, use fal.ai's BRIA endpoint (cloud, see below).

### Cloud API providers (optional, bring your own key)

Local is the default. Cloud is opt-in — your images are only sent when you explicitly enable cloud mode in Settings, and your API key stays on your Mac.

**Shipped today:**

| Provider                              | Cost                  | Endpoint                                 | Notes                                                          |
| ------------------------------------- | --------------------- | ---------------------------------------- | -------------------------------------------------------------- |
| **Replicate**                         | Variable              | Community BiRefNet model                 | Pay-per-run or compute-time; check the model page              |
| **fal.ai BiRefNet**                   | Variable              | `fal-ai/birefnet`                        | Fast and reliable                                              |
| **fal.ai BRIA RMBG 2.0**              | Variable              | `fal-ai/bria/background/remove`          | Commercial-safe RMBG — avoids the local non-commercial license |
| **fal.ai Ideogram Remove Background** | Variable              | `fal-ai/ideogram/remove-background`      | High-quality cutouts with clean edges                          |
| **remove.bg**                         | Paid credits          | Proprietary                              | Mature API; useful as quality benchmark                        |
| **Photoroom**                         | Paid credits          | `sdk.photoroom.com/v1/segment`           | Strong product-photo workflow for ecommerce users              |

When fal.ai is selected, pick the endpoint in **Settings → Cloud → Endpoint**. See [`docs/TODO.md`](docs/TODO.md) Phase 10 for what landed.

## Installation

### Install from DMG

DropBG does not yet have an Apple Developer certificate, so macOS will block the app on first launch:

1. Try to open DropBG — macOS shows a warning
2. Go to **System Settings → Privacy & Security**
3. Click **Open Anyway** next to the DropBG message
4. Or run: `xattr -cr /Applications/DropBG.app`

See [docs/USAGE.md](docs/USAGE.md) for detailed instructions.

### Build from source

> Prerequisites: [Rust toolchain](https://rustup.rs/), [Bun](https://bun.sh/)

```bash
# clone
git clone https://github.com/whereissam/DropBG && cd DropBG

# install frontend dependencies
bun install

# run in dev mode
cargo tauri dev

# or build a release DMG
cargo tauri build
```

The DMG/app bundle will be in `src-tauri/target/release/bundle/`.

## Usage

1. **Launch DropBG.** On first run, download an AI model (~200 MB).
2. **Drop an image** (or several) onto the window.
3. **Save** the result. Optionally swap in a new background or upscale.

That's it. Everything after the initial model download runs offline.

## Current Limitations

- First launch requires downloading a model
- Local speed depends on your Mac and selected model
- Very large images may use significant memory
- Some model weights have separate licenses from the app
- Unsigned macOS builds require manual approval on first launch

## Tech Stack

| Layer        | Choice                              | Why                                          |
| ------------ | ----------------------------------- | -------------------------------------------- |
| App          | **Tauri 2** (Rust + React)          | Lightweight, native feel                     |
| AI inference | **ort** (ONNX Runtime) + CoreML EP  | Optimized for Apple Silicon, CoreML where available |
| AI upscale   | **Real-ESRGAN x4plus** (ONNX)       | High-quality super-resolution                |
| Image        | **image** crate + **ndarray**       | PNG read/write with alpha channel            |
| Frontend     | **React 19** + TypeScript           | Fast, component-based UI                     |

## Roadmap

- More polished model picker with on-device benchmarks
- Smart subject selection (SAM-style) for manual touch-ups
- Optional automatic alpha matting refinement
- Signed/notarized macOS builds

See [docs/TODO.md](docs/TODO.md) for the full roadmap.

## Project Structure

```
DropBG/
├── src-tauri/                # Rust backend
│   ├── src/
│   │   ├── lib.rs            # Tauri entry + command registration
│   │   ├── commands.rs       # IPC command handlers
│   │   ├── inference/        # ONNX Runtime session, pre/post processing, upscale
│   │   ├── imaging/          # Auto-crop, background replacement
│   │   └── model/            # Model downloader + config management
│   └── Cargo.toml
├── src/                      # React frontend
│   ├── App.tsx               # Main app (stage-based routing)
│   ├── tauri.ts              # Typed Tauri invoke wrappers
│   └── components/           # UI components
├── web/                      # Landing page (Astro)
├── docs/
│   ├── USAGE.md              # Usage guide
│   └── TODO.md               # Roadmap
└── README.md
```

## Documentation

- [Usage Guide](docs/USAGE.md) — installation, features, models, troubleshooting
- [TODO / Roadmap](docs/TODO.md) — development phases and progress

## License

DropBG is MIT-licensed.

AI model weights downloaded by DropBG are **not** covered by this license. Each model has its own license — see the [Models](#models) section above. In particular, RMBG 2.0 is non-commercial unless you have a commercial agreement with BRIA.
