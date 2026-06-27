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
- [x] Model selector in Settings (multi-model: BiRefNet Lite, BiRefNet Full, BEN2, InSPyReNet, MODNet)
- [x] Auto-download selected model variant
- [x] Allow switching models without restarting (hot-swap via session clear)
- [x] BEN2 (219 MB) — best on hair & fine edges, MIT license
- [x] MODNet (13 MB) — lightweight, optimized for portraits & people
- [x] BiRefNet Portrait (490 MB) — specialized portrait/people model, modern MODNet replacement
- [x] BiRefNet General (490 MB) — newer training epoch 244, improved general quality

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
- [x] DMG installer build (DropBG_1.0.0_aarch64.dmg, 16 MB)
- [x] Landing page with how-it-works, models, and install guide
- [x] Usage guide (docs/USAGE.md)
- [x] Version bump to 1.0.0
- [x] Onboarding guide (permissions, unsigned app workaround)
- [x] RMBG 2.0 manual download flow (gated HuggingFace model)
- [ ] macOS code signing & notarization
- [ ] GitHub releases with DMG attached
- [ ] Performance benchmarks on Apple Silicon (M1/M2/M3)

## Phase 8: Cloud API & Model Expansion

- [x] Add InSPyReNet model (SwinB Plus Ultra, ~300 MB, best on fuzzy edges/hair strands)
- [x] Cloud API provider framework (Replicate, fal.ai, remove.bg)
- [x] API key management in Settings (local config file)
- [x] Cloud/local toggle — switch between local inference and cloud API
- [x] Replicate integration (community background-removal models on GPU, variable pay-per-run pricing)
- [x] fal.ai integration (BiRefNet + BRIA RMBG 2.0, ~$0.018/generation as of 2026-05; check provider page)
- [x] remove.bg integration (proprietary model, paid credits)
- [x] Skip model download when cloud mode enabled
- [x] Cloud batch processing
- [x] Usage tracking / cost estimation per session

## Phase 9: Product Polish Pass

A focused pass to bring the product surface (landing page, in-app UX, docs, code) up to the same trust/positioning standard as the rewritten README. Work top-to-bottom; each step lands as its own commit.

### 9.1 — Landing page consistency (`web/src/pages/index.astro`) ✅

- [x] Hero subtitle: drop fragile "9 AI models" claim, mirror new README pitch
- [x] Stats row: replace "9+ AI models" with a non-fragile metric (`MIT / Open source`)
- [x] Insert "Why DropBG vs cloud tools" comparison table (between Stats and Features)
- [x] Models table: add License column (Status reflected via "Default" tag); mobile cards in sync
- [x] Add footnote on RMBG 2.0 (CC BY-NC 4.0; commercial requires BRIA agreement)
- [x] Add an "Honest limitations" section before the CTA
- [x] Install card: stop hardcoding `DropBG_1.0.0_aarch64.dmg` and `16 MB` — link to `/releases/latest`
- [x] Add cloud-API section (BYO key, opt-in framing, dated fal.ai price only)
- [x] Add `#cloud` link to desktop nav so the new section is reachable

### 9.2 — Docs + marketing repositioning (curated lineup)

Sharpened lineup based on product feedback (see `MODEL_RESEARCH.md` for rationale). README, landing page, and USAGE should all tell the same story:

**Local lineup to surface prominently:**
- Apple Vision (default, zero download)
- BiRefNet Lite (recommended local download)
- BiRefNet General (quality)
- BiRefNet Matting / Dynamic (advanced; manual download)
- BEN2 (best edges; experimental)
- RMBG 2.0 (product; **non-commercial license** unless agreement with BRIA)
- MODNet (legacy; lightweight)

**Local lineup to de-emphasize in marketing tables (keep working in app):**
- BiRefNet Portrait (subsumed by Matting/Dynamic guidance)
- InSPyReNet (advanced; manual; move below the fold)

**Cloud lineup to surface (currently shipped):**
- Replicate (community models, variable pricing)
- fal.ai BiRefNet endpoint (fast, reliable)
- remove.bg (mature, paid credits, quality benchmark)

**Cloud lineup roadmap (not yet shipped — see Phase 10):**
- fal.ai Ideogram Remove Background
- fal.ai BRIA RMBG 2.0 (commercial-safe RMBG via API)
- Photoroom API

Tasks:
- [x] `README.md` — rewrite local + cloud tables to the curated lineup with license column
- [x] `web/src/pages/index.astro` — update models table + cloud cards to match (shipped + roadmap split)
- [x] `docs/USAGE.md` — Apple Vision as default; curated models with license column; Cloud APIs section; strengthened RMBG 2.0 callout; ViTMatte two-stage note; Troubleshooting with Apple Vision fallback
- [x] Mark not-yet-shipped models/providers explicitly as "Coming soon / on the roadmap" in all three surfaces

## Phase 10 — Model & Cloud Lineup Expansion

Implementation work to back the curated docs lineup. Each item adds a download URL or a cloud endpoint behind the existing abstractions; no UI refactor required.

### 10.1 — Local model expansion ✅

- [x] Added `ModelVariant::HRMatting` → `ZhengPeng7/BiRefNet_HR-matting` (2048×2048 trained)
- [x] Pattern: manual download via export script (matches existing Matting / Dynamic flow — no pre-built ONNX upstream)
- [x] Wired through `downloader.rs`: `name`, `filename` (`birefnet_hr_matting_fp16.onnx`), `manual_download_url`, `requires_manual_download`, `approx_size` (~900 MB), `description`, `input_size` (2048), `is_matting_model` (now true for both Matting and HRMatting), `variant_key`, `from_key`
- [x] Added `scripts/export_hr_matting_onnx.py` (mirrors `export_matting_onnx.py` with INPUT_SIZE=2048, MODEL_ID=`ZhengPeng7/BiRefNet_HR-matting`)
- [x] Settings.tsx: export-script branch now handles `HRMatting`; renders the appropriate `python scripts/export_hr_matting_onnx.py` command and a 2048×2048 memory note
- [x] Docs synced: README + landing page + USAGE.md tables list HR-matting; USAGE Notes section warns about ~4× memory use
- [x] Kept BiRefNet Matting (the lite variant) — it's still useful for fast hair/fur mattes; the two coexist rather than rename

### 10.2 — fal.ai endpoint expansion ✅

- [x] Added `FalAIEndpoint` enum (BiRefNet / BriaRmbg / Ideogram) in `downloader.rs`
- [x] Persisted `fal_ai_endpoint` in `AppConfig`
- [x] Refactored `inference/cloud.rs:fal_remove_bg` to dispatch URL by endpoint (all three endpoints share the same `image_url → image.url` shape — verified via fal.ai docs)
- [x] Extended `get_cloud_config` to return `fal_ai_endpoint`, `fal_ai_endpoint_name`, `fal_ai_endpoints[]`
- [x] Added `set_fal_ai_endpoint` Tauri command, registered in `lib.rs`
- [x] Added `setFalAiEndpoint` + types in `tauri.ts`
- [x] Settings.tsx: endpoint sub-picker appears under the provider picker only when fal.ai is active; BriaRmbg shows a "Commercial-safe" tag
- [ ] **Follow-up:** `cloud_usage.rs` cost estimate is per-provider, not per-endpoint. BRIA RMBG (~$0.018/gen) and Ideogram have different prices than BiRefNet — could improve fidelity by tracking endpoint-level costs. Low priority since the user-facing copy already directs to the provider page for accurate pricing.

### 10.3 — Photoroom API integration ✅

- [x] Added `CloudProvider::Photoroom` to the enum (`name`, `variant_key`, `from_key`, `description`, included in `all()`)
- [x] API key field in Settings — flows through existing per-provider `cloud_api_keys` HashMap; pricing link points to `docs.photoroom.com/getting-started/pricing`
- [x] Endpoint wired in `inference/cloud.rs:photoroom_remove_bg` — `POST sdk.photoroom.com/v1/segment`, `x-api-key` header, multipart `image_file` field, raw PNG response (verified via Photoroom OpenAPI docs)
- [x] Cost tracking in `cloud_usage.rs` (added `photoroom: u32` counter, default $0.10/img placeholder)
- [x] Docs synced: README + landing page + USAGE.md all list Photoroom under "Shipped today"; landing page consolidated to a single 6-card grid (no more roadmap section since Phase 10 is now fully landed)

### 10.4 — Cloud lineup polish

- [x] "Commercial-safe" badge for fal.ai BRIA RMBG 2.0 — appears on the endpoint sub-picker in Settings (see 10.2)
- [x] Per-provider link-out to the provider's pricing page from Settings (`CLOUD_PRICING_URLS` in `Settings.tsx`, covers all four providers)
- [ ] Optional: same "commercial-safe" badge for remove.bg / Photoroom — both are commercial APIs by design; lower priority

### 9.3 — In-app UX polish (Tauri frontend) ✅

- [x] First-run / onboarding (`Onboarding.tsx`): killed "must download a model" framing; "Works Out of the Box" with Apple Vision
- [x] First-run / model setup (`ModelSetup.tsx`): primary CTA is "Get Started with Apple Vision" when available; download moved to secondary
- [x] Model picker (`Settings.tsx`): license tags (MIT / Apache 2.0 / Non-commercial) per model, with warn tone for non-commercial
- [x] Cloud key flow (`Settings.tsx`): added "Your key stays on this Mac" framing + per-provider "View pricing" link-out
- [x] Cloud provider descriptions (`downloader.rs`): replaced fragile "$0.0004/img" pricing with non-fragile descriptions
- [x] ~~Batch progress: surface per-image error states~~ — already implemented in `BatchList.tsx:51`
- [x] ~~Toolbar empty states~~ — N/A (Toolbar only renders when result exists; empty state lives in `DropZone`)
- [ ] Settings.tsx (689 lines) — split into sub-panels (Model, Upscale, Cloud, Storage). **Deferred** to a focused refactor session — see Phase 9.4.

### 9.4 — Code quality / tech debt

- [x] `src-tauri/src/commands.rs` (1020 lines) — split into a 50-line parent module + 6 focused submodules under `commands/` (system, model, inference, cloud, editing, postprocess). Shared `ProcessProgress` / `BatchProgress` / `emit_progress` live in the parent. `lib.rs` registrations unchanged via `pub use commands::<sub>::*`. `cargo check` ✅ / `cargo test --lib` 10/10 ✅.
- [ ] **Deferred (own session):** Typed error surface for `tauri.ts` invoke wrappers — replace `Result<T, String>` with a discriminated union (`DropbgError` enum on Rust side, mirrored TS type). Touches every command and every frontend invoke site.
- [x] ~~Remove dead/legacy paths flagged by MODNet "legacy" status~~ — No dead code found. MODNet is fully wired; "legacy" is intentional user-facing copy. Also: deferred Settings.tsx 689-line split belongs here too as its own refactor session.
- [x] ~~Audit `unwrap()` / `expect()` in Rust modules~~ — Audited: only 1 `.expect()` in the entire backend (`lib.rs:62`, Tauri `app.run()` boilerplate where panic is correct). Codebase is already clean.
- [x] Add a minimal smoke-test harness — added `#[cfg(test)] mod tests` to `imaging/autocrop.rs` (5 tests) and `imaging/background.rs` (5 tests). `cargo test --lib` passes 10/10 in ~0.00s. Foundation for catching regressions in pure imaging logic.

## Phase 11 — Quality Engine Pass (post-lineup)

Roadmap based on a model + performance review dated **2026-06-25**. Conclusion:
the curated lineup is **not** outdated — no MIT-licensed, easily ONNX-able model
has emerged that broadly beats BiRefNet HR-matting. The one obvious gap is
**BiRefNet Dynamic Matting**. Beyond that, the next real product jump comes from
the *inference + post-processing* layer (Native Core ML, FP16, on-device
benchmarking, edge-only HR refinement, foreground decontamination), **not** from
shipping a 12th model. Research models (DiffDIS, PDFNet, depth-assisted DIS) stay
in a benchmark backlog — too heavy, ONNX/Core ML port risk too high, alpha not
necessarily compositing-grade.

Work top-to-bottom; steps are ordered by impact. Each lands as its own commit.

### 11.1 — Add BiRefNet Dynamic Matting (new Quality default)

`ZhengPeng7/BiRefNet_dynamic-matting` — distinct from the existing `Dynamic`:
Dynamic is fg/bg segmentation, Dynamic-matting outputs a finer **alpha matte**,
accepts arbitrary ~256–2304 px sizes/aspect ratios (no forced square resize that
blurs hair / drops thin product edges).

- [x] Add `ModelVariant::DynamicMatting` in `downloader.rs` (name, filename `birefnet_dynamic_matting_fp16.onnx`, `is_matting_model = true`, `is_dynamic = true`, `input_size = 0` native, `requires_manual_download`, `manual_download_url` → `ZhengPeng7/BiRefNet_dynamic-matting`, `approx_size` ~490 MB, `variant_key`, `from_key`). `cargo check` ✅
- [x] Add `scripts/export_dynamic_matting_onnx.py` (mirrors `export_dynamic_onnx.py`, MODEL_ID `ZhengPeng7/BiRefNet_dynamic-matting`, dynamic H/W axes, fp16) + Settings.tsx export-script branch handles `DynamicMatting`
- [ ] **Validate first:** confirm dynamic-shape ONNX export does not force a Core ML EP → CPU fallback before promoting it as default (see 11.2 benchmark)
- [ ] Promote Dynamic Matting to the **Quality default**, demoting `General` to an advanced option — **blocked on the validation above**; shipped as a manual/advanced model for now
- [x] Docs sync (README / landing / USAGE) — added to lineup tables with MIT license column

### 11.2 — Inference backend selection (Native Core ML + FP16 + auto-benchmark)

ORT's Core ML EP can use CPU/GPU/Neural Engine, but unsupported ops get
partitioned back to CPU and the partitioning overhead can make it *slower* than
plain CPU. Don't assume Native Core ML is faster either — measure per machine.

**11.2a — Backend abstraction + on-device benchmark (shipped):**
- [x] `inference/backend.rs` — `Backend` enum (`CoreMlEp` / `Cpu`), centralized `build_session(path, backend)` (replaces the two duplicated builders in `session.rs`), `device_id()` (macOS `hw.model`), `resolve_backend()` (persisted winner or default), `backend_info()`
- [x] Backend abstraction with the two ORT paths that exist today: ORT CPU · ORT Core ML EP (Native Core ML is a planned third candidate — see 11.2b)
- [x] On-device micro-benchmark: warm-up + median over 3 runs per backend on a deterministic synthetic input; compares output to the CPU reference and **rejects a faster backend whose output diverges** (mean-abs-diff > 0.05); persists the fastest *correct* backend per `{variant}:{device}` in `AppConfig.backend_benchmarks`
- [x] `get_backend_info` + `benchmark_inference_backends` Tauri commands (registered in `lib.rs`); `tauri.ts` wrappers + types; Settings → **Inference Backend** section (shows this Mac, current backend, per-backend timings, "Benchmark Backends" button). `cargo check` ✅ / `cargo test --lib` 10/10 ✅ / `bun run build` ✅
- [x] This is the validation gate for promoting Dynamic Matting (11.1): if the dynamic-shape ONNX makes the Core ML EP partition to CPU and run slower, the benchmark now picks CPU and flags it instead of silently regressing

**11.2b — Native Core ML + FP16 policy (deferred — heavier, hardware-validated):**
- [ ] Add a Native Core ML backend: ship/convert each curated model to FP16 `.mlpackage`, compile to `.mlmodelc` on first use; add it as a third `Backend` candidate so the benchmark picks among all three
- [ ] Add peak-memory measurement to the benchmark report (latency + output-diff are in; memory is not yet captured) and extend the persisted record toward `{ median_ms, peak_memory_mb, precision }`
- [ ] FP16 policy: Apple Silicon default FP16; Intel Mac benchmark FP16 vs FP32; keep mask resize / normalization / compositing in FP32 to avoid alpha banding
- [ ] Once 11.2a/11.2b confirm Dynamic Matting's backend is a win, promote it to the Quality default (closes the held item in 11.1)
- [x] (Supersedes Phase 7 "Performance benchmarks on Apple Silicon" — folded in here)

### 11.3 — Model picker → 4 user modes

Stop leading the UI with ~10 technical model names. Surface four modes; expose
raw model selection only under Advanced.

- [x] `ProcessingMode` enum in `downloader.rs` (Fast / Balanced / BestEdges / Product / Advanced) with `variant()` mapping + `uses_apple_vision()`; persisted in `AppConfig.processing_mode` (defaults to Advanced so existing configs are unchanged); `get_processing_mode` / `set_processing_mode` commands; `set_model_variant` now drops to Advanced when a raw model is picked
- [x] **Fast** → Apple Vision (App.tsx routes Fast → `removeBackgroundAppleVision`; falls back to the downloaded model / setup flow when Vision is unavailable)
- [x] **Balanced** → BiRefNet Dynamic Matting
- [x] **Best Edges** → BiRefNet HR-matting *(auto edge-refinement pass is wired in 11.4, not yet auto-applied by the mode)*
- [x] **Product** → BiRefNet Dynamic *(hard-edge cleanup pass deferred to 11.5)*
- [x] Settings leads with 4 mode cards; the full model list is collapsed under an **Advanced — choose a specific model** disclosure (RMBG 2.0 stays in Advanced only, never a mode default)
- [~] Per-mode card surfaces the *measured* backend from 11.2 (e.g. "Backend: Core ML…") once benchmarked. Full `1.8 s · 1.2 GB · Neural Engine` line + per-image "Recommended" hint still pending — needs the benchmark to record latency/memory per backend (11.2b) and a per-image heuristic

### 11.4 — Edge-only HR refinement (two-stage, tiled)

Don't run a heavy model over a whole 6000×4000 image, and don't downscale
everything either. Coarse mask first, then HR-matting only on uncertain edges.

Shipped as `inference/hr_refine.rs` + the `refine_edges_hr` command, exposed as an
opt-in **"HR Edges"** button in the Toolbar (mirrors the ViTMatte "Refine" post-step,
so the default pipeline is untouched).

- [x] Stage 1: reuse the coarse mask from the already-produced cutout (alpha channel of the result)
- [x] Detect uncertain regions: soft-alpha band (16–240), binary-dilated by 12 px, then box-blurred into a smooth 0..1 blend weight (kills the seam between coarse and refined)
- [x] Stage 2: 512 px tiles with 128 px overlap, **only over tiles that contain uncertain pixels**; each tile is upscaled to HR-matting's 2048² input, run, and the alpha resized back; feathered (linear-ramp window) overlap-blend into a full-res accumulator
- [x] Composite: `final = coarse·(1−u) + refined·u` so confident interior/background keeps the coarse alpha and only the edges get the heavy model
- [x] Memory-friendly: one tile session reused across tiles, peak ≈ a single 2048² forward pass + two f32 full-res accumulators (not a full-image HR pass); errors clearly if HR-matting isn't downloaded
- [x] `cargo check` ✅ / `cargo test --lib` 10/10 ✅ / `bun run build` ✅
- [ ] **Follow-up (needs on-device tuning):** confirm tile/overlap/band constants on real hair/fur images; consider an "auto-apply in Best Edges mode" hook (11.3) once validated

### 11.5 — Foreground decontamination + 16-bit alpha

Background removal isn't just an alpha mask — hair shot on blue/green/dark
backgrounds leaves colored fringes when composited onto white.

Shipped as `imaging/decontaminate.rs` + the `decontaminate_result` command, exposed
as an opt-in **"Decontaminate"** Toolbar action and a **Save → 16-bit** option.

- [x] Foreground color estimation + edge decontamination: alpha²-weighted color diffusion floods true foreground color from the opaque core into the soft band, suppressing the `(1−α)·B` background contribution (the green/blue hair fringe). Alpha is left untouched; only edge-band color changes. Unit-tested (4 tests, incl. "edge pulled toward foreground").
- [x] Optional **16-bit PNG export**: the decontaminated foreground color is encoded straight from the floating-point estimate (no re-quantization banding), saved via the existing raw-bytes `save_image` path (Save → 16-bit).
- [ ] **Follow-up:** alpha precision is still bounded by the 8-bit inference mask (16-bit alpha gains full benefit only once the f32 mask is threaded end-to-end — overlaps 11.2b). Also consider auto-running decontamination as the final step of Best Edges / Product modes (11.3 hook).

### 11.6 — Internal benchmark set + copy fixes

- [x] Benchmark set **scaffold + protocol**: `docs/BENCHMARK.md` (categories + targets, eval protocol, scoring on white *and* mid-gray, what results decide) and a `bench/` directory (8 category folders, `manifest.csv` + `results.csv` templates, `.gitignore` that keeps structure but excludes the private images). Populating it with real images is a manual data-collection task — can't be code-generated.
- [x] Reworded BEN2 copy to "Experimental alternative for difficult boundaries — benchmark against BiRefNet Matting first" across all four surfaces: README + USAGE (done in 11.1) + landing page (`index.astro`, both mobile + desktop) + `downloader.rs` description
- [x] DiffDIS / PDFNet / depth-assisted DIS (+ SAM 3) documented in a **Research Backlog** section of `MODEL_RESEARCH.md` with per-model rationale; confirmed none are in the `ModelVariant` picker

## Stretch Goals

- [ ] Figma plugin companion (thin plugin → DropBG local API)
- [x] Auto model routing (YuNet face detection → BiRefNet Portrait when faces detected)
- [x] BiRefNet-matting support (true alpha mattes for hair/fur/transparency, export script provided)
- [x] BiRefNet Dynamic support (native resolution 256-2304px, no resize artifacts, export script provided)
- [x] Two-stage pipeline: coarse mask (BiRefNet) → refined alpha (ViTMatte Small, 28 MB)
- [ ] Video background removal (frame-by-frame)
- [ ] Windows / Linux support
