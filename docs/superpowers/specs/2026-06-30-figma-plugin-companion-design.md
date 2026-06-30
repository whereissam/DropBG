# Figma Plugin Companion — Design Spec

**Date:** 2026-06-30
**Status:** Approved (brainstorm), pending implementation plan
**Scope:** v1 — single action (remove background), dev/personal distribution.

## Goal

Let a user remove an image's background from inside Figma using their local
DropBG app, with no upload to any cloud service. The plugin is a thin client; all
inference happens in the already-running DropBG desktop app over a localhost HTTP
API.

## Non-goals (v1)

- Background replacement, upscaling, auto-crop, decontaminate (DropBG has these,
  but they are out of scope for v1 — see Future).
- Publishing to the Figma Community. v1 runs as a local development plugin only.
- Auto-launching DropBG. If the app isn't running, the plugin shows a clear
  message; it does not try to start it.

## Architecture overview

```
Figma (desktop/web)                      DropBG.app (Tauri, already running)
┌─────────────────────────┐              ┌──────────────────────────────────┐
│ code.js (main thread)   │              │ tiny_http server thread           │
│  - read selection       │   bytes      │  127.0.0.1:8765                   │
│  - exportAsync PNG ──────┐             │   GET  /v1/health                 │
│  - apply result         │ │            │   POST /v1/remove                 │
│        ▲                 │ │ postMsg    │        │                          │
│        │ postMsg         ▼ │            │        ▼                          │
│ ui.html (iframe)        ───┼─ fetch ───▶│  run_removal(img, &SessionState…) │
│  - fetch localhost      │              │   (shared, already-loaded model)  │
│  - health check         │              └──────────────────────────────────┘
└─────────────────────────┘
```

Only the `ui.html` iframe can `fetch`; only `code.js` can touch the Figma
document. They communicate via `postMessage`.

## Component 1 — DropBG local HTTP API (Rust)

### Server lifecycle
- New module `src-tauri/src/api_server.rs`.
- Started from a dedicated OS thread spawned in `lib.rs` `run()` after the
  Tauri builder's managed state is set up (via `.setup(|app| { … })`), so the
  handler can resolve managed state through the `AppHandle`.
- Uses `tiny_http` (new dependency — small, blocking, no async runtime needed on
  its thread). Chosen over `axum` to keep the release binary small (`opt-level
  = "s"`) and avoid entangling the server with the app's tokio runtime.
- Binds to **`127.0.0.1:8765` only** (loopback; never `0.0.0.0`). If the port is
  taken, log a warning and skip starting the server — the desktop app still
  works normally.
- **Serial request handling.** The server runs a single accept loop and
  processes one request at a time (no thread pool). The ONNX session is shared
  mutable state; serializing requests avoids two concurrent Figma calls
  contending for the model or doubling inference memory. Adequate for a
  single-user personal plugin.

### Endpoints

`GET /v1/health`
- Response `200`, JSON: `{ "ok": true, "model": "<loaded variant or 'none'>" }`.
- Used by the plugin on open to confirm DropBG is reachable.

`POST /v1/remove`
- Request body: **raw image bytes** (PNG or JPEG). Content-Type ignored;
  decoded with `image::load_from_memory`.
- **Body size limit: 50 MB.** A larger `Content-Length` (or a body that exceeds
  the cap while reading) is rejected with `413` before decoding, so a giant
  Figma frame export can't stall the server thread or blow up inference memory.
- Response `200`: **raw cutout PNG bytes**, `Content-Type: image/png`.
- Errors: `400` for undecodable/empty body, `413` for oversize body, `500` with
  a plain-text message for inference failure, `503` if the model isn't
  downloaded/loaded yet.

`OPTIONS` (any path)
- Returns `204` with CORS headers for preflight.

### CORS
Every response includes:
```
Access-Control-Allow-Origin: *
Access-Control-Allow-Methods: GET, POST, OPTIONS
Access-Control-Allow-Headers: Content-Type, Authorization
```
Required because a Figma plugin iframe's origin is `null`. `*` is acceptable
because the API is non-credentialed and loopback-only.

### Engine reuse
The existing `commands::inference::process_single_image(&SessionState, &PathBuf,
mask_size)` already performs path → PNG-bytes with no `AppHandle`/progress
dependency. Refactor its core to decode from bytes:

```
fn process_image_bytes(
    session_state: &SessionState,
    bytes: &[u8],
    mask_size: u32,
) -> Result<Vec<u8>, String>
```

- `remove_background` (Tauri command) and `remove_background_batch` keep their
  current behavior; `process_single_image` becomes a thin wrapper that reads the
  file and calls `process_image_bytes`.
- The HTTP handler resolves `app.state::<SessionState>()`, calls
  `state.ensure_loaded()`, reads `mask_size` from the active model config, and
  calls `process_image_bytes`. **The model is loaded once and shared** — no
  duplicate session, no extra memory.
- v1 uses the user's currently-selected model. Auto-routing (face → Portrait)
  and the f32-alpha hi-res cache are **not** wired into the HTTP path in v1
  (kept simple; see Future).

### Security
- Loopback-only bind is the primary control.
- Optional `DROPBG_API_TOKEN` env var: if set, every `/v1/*` request must send
  `Authorization: Bearer <token>`; otherwise `401`. If unset (default for
  personal use), no auth is required.
- Documented residual risk: with no token, another local process could ask the
  app to remove a background. Low impact (no data exfiltration, no file access).

## Component 2 — Figma plugin (TypeScript/JS)

New directory `figma-plugin/` in the repo (not bundled into the app).

### Files
- `manifest.json`:
  ```json
  {
    "name": "DropBG Local",
    "id": "dropbg-local-dev",
    "api": "1.0.0",
    "editorType": ["figma"],
    "main": "code.js",
    "ui": "ui.html",
    "documentAccess": "dynamic-page",
    "networkAccess": {
      "allowedDomains": ["none"],
      "devAllowedDomains": [
        "http://127.0.0.1:8765",
        "http://localhost:8765"
      ]
    }
  }
  ```
  Per Figma docs, a local/development server belongs in **`devAllowedDomains`**;
  putting it in `allowedDomains` would require a mandatory `reasoning` field and
  is only needed for a published plugin. Both `127.0.0.1` and `localhost` are
  listed so host-resolution differences between Figma desktop and web don't
  break the fetch. (When/if this is published, move the host to `allowedDomains`
  with a `reasoning` string — see Future.)
- `code.js` — main thread (Figma API; cannot `fetch`).
- `ui.html` — **visible** iframe (`figma.showUI(__html__, { width: 320, height:
  160 })`), can `fetch`; shows connection status + a "Remove background" button.
  Visible UI (rather than hidden) keeps error states debuggable and makes the
  plugin feel like a tool.

### API base resolution
`ui.html` tries bases in order — `http://127.0.0.1:8765` first, then
`http://localhost:8765` — and remembers the first that answers `/v1/health`, so
host-resolution quirks don't break the round trip.

### Message contract (fixed shapes — never pass bare bytes)
```ts
type PluginToUi =
  | { type: "health-check" }
  | { type: "remove"; requestId: string; bytes: Uint8Array; token?: string };

type UiToPlugin =
  | { type: "health-ok"; model: string }
  | { type: "health-error"; message: string }
  | { type: "remove-ok"; requestId: string; bytes: Uint8Array }
  | { type: "remove-error"; requestId: string; status?: number; message: string };
```

### Flow
1. On launch, `code.js` shows the UI; `ui.html` runs API-base resolution against
   `GET /v1/health`.
   - Reachable → status "DropBG connected (model: X)".
   - Connection refused → status "Start DropBG and reopen this plugin."
2. User selects a node and clicks "Remove background".
3. `code.js` validates the selection (see Selection scope below). On failure,
   `figma.notify` a clear message and stop.
4. `code.js` calls `node.exportAsync({ format: 'PNG' })` → `Uint8Array`, and
   posts `{ type: "remove", requestId, bytes }` to `ui.html`.
5. `ui.html` POSTs the bytes to `<base>/v1/remove`.
   - Network error → post `remove-error` "Is DropBG running?".
   - Non-200 → post `remove-error` with status + the server's error text.
   - 200 → post `remove-ok` with the cutout bytes (matched by `requestId`).
6. `code.js` applies **Replace + keep original hidden**:
   - `const backup = node.clone()` — the clone retains the original fills.
   - `backup.visible = false`; `backup.name = node.name + " (original)"`.
   - Replace **only the first `IMAGE` fill** on the visible `node` with
     `figma.createImage(cutoutBytes)` (`scaleMode: 'FILL'`), preserving the
     node's size and any non-image fills/effects. (Wholesale fill replacement
     would drop solid overlays/effects — avoided.)
   - Reselect `node`; notify "Background removed". Undo reverts in one step.

### Selection scope (v1)
v1 processes **exactly one selected node that has at least one `IMAGE` fill.**
No fallback to "exportable as image" for arbitrary nodes — exporting a
vector/text/frame/group and stuffing it back as an image fill changes semantics
too much and is a reliability trap. Broader node support is a later iteration.

### Edge cases
| Condition | Behavior |
|-----------|----------|
| No selection | "Select an image layer first." |
| Multiple nodes selected | "Select exactly one image layer." |
| Node has no IMAGE fill | "This layer has no image fill to process." |
| Server unreachable | "Start DropBG and try again." |
| Body too large (413) | "Image is too large (max 50 MB export)." |
| Model not downloaded (503) | "Open DropBG and download a model first." |
| Server 500 | Show the server's error text. |

## Testing

- **Rust:** unit test `process_image_bytes` (decode a small fixture, assert RGBA
  PNG out with an alpha channel). Manual: `curl -i
  http://127.0.0.1:8765/v1/health`, `curl --data-binary @in.png
  http://127.0.0.1:8765/v1/remove -o out.png`, `curl -i -X OPTIONS
  http://127.0.0.1:8765/v1/remove` (CORS preflight), and an oversize body to
  confirm `413`.
- **Plugin:** manual in Figma — load as a development plugin, run against the
  full matrix in the edge-case table (happy path + each error row). No automated
  Figma test harness in v1.
- Regression: confirm `cargo test --lib`, `cargo check`, and `bun run build`
  stay green; existing `remove_background` / batch behavior unchanged.

## Biggest risk
Not the Rust or inference — it's **Figma fill semantics**. A rectangle with an
image fill replaces cleanly; groups/frames/vectors/instances/masks or nodes with
mixed fills don't. v1 deliberately narrows to "one node with an IMAGE fill,
replace the first image fill in place" so the plugin is reliable instead of
magical-but-buggy.

## Future (explicitly out of v1)
- Additional endpoints: background replace, upscale, auto-crop, decontaminate.
- Wire auto-routing and the f32-alpha hi-res cache into the HTTP path.
- Broader node support (export arbitrary nodes, multi-selection).
- Publish to the Figma Community: move the host from `devAllowedDomains` to
  `allowedDomains` **with a required `reasoning` string**, plus icons,
  screenshots, listing, and review.
- Make the port configurable in DropBG settings.
