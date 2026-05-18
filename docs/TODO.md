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

- [ ] **Deferred (own session):** `src-tauri/src/commands.rs` (1020 lines) — split into `commands/` directory with `mod.rs` + sub-modules (cloud / model / inference / editing / system). High-blast-radius mechanical refactor; needs `cargo check` after each move.
- [ ] **Deferred (own session):** Typed error surface for `tauri.ts` invoke wrappers — replace `Result<T, String>` with a discriminated union (`DropbgError` enum on Rust side, mirrored TS type). Touches every command and every frontend invoke site.
- [x] ~~Remove dead/legacy paths flagged by MODNet "legacy" status~~ — No dead code found. MODNet is fully wired; "legacy" is intentional user-facing copy. Also: deferred Settings.tsx 689-line split belongs here too as its own refactor session.
- [x] ~~Audit `unwrap()` / `expect()` in Rust modules~~ — Audited: only 1 `.expect()` in the entire backend (`lib.rs:62`, Tauri `app.run()` boilerplate where panic is correct). Codebase is already clean.
- [x] Add a minimal smoke-test harness — added `#[cfg(test)] mod tests` to `imaging/autocrop.rs` (5 tests) and `imaging/background.rs` (5 tests). `cargo test --lib` passes 10/10 in ~0.00s. Foundation for catching regressions in pure imaging logic.

## Stretch Goals

- [ ] Figma plugin companion (thin plugin → DropBG local API)
- [x] Auto model routing (YuNet face detection → BiRefNet Portrait when faces detected)
- [x] BiRefNet-matting support (true alpha mattes for hair/fur/transparency, export script provided)
- [x] BiRefNet Dynamic support (native resolution 256-2304px, no resize artifacts, export script provided)
- [x] Two-stage pipeline: coarse mask (BiRefNet) → refined alpha (ViTMatte Small, 28 MB)
- [ ] Video background removal (frame-by-frame)
- [ ] Windows / Linux support
