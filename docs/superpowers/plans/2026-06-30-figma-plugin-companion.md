# Figma Plugin Companion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let a user remove an image's background from inside Figma by calling the already-running DropBG desktop app over a localhost HTTP API.

**Architecture:** A small `tiny_http` server thread inside the Tauri app exposes `GET /v1/health` and `POST /v1/remove` on `127.0.0.1:8765`, reusing the loaded ONNX session via Tauri managed state. A thin dev-only Figma plugin exports the selected image node, POSTs the bytes, and swaps the returned cutout into the node's first IMAGE fill (keeping the original as a hidden backup).

**Tech Stack:** Rust (Tauri 2, `tiny_http`, `ort`, `image`), Figma Plugin API (vanilla JS).

**Spec:** `docs/superpowers/specs/2026-06-30-figma-plugin-companion-design.md`

## Global Constraints

- Server binds **`127.0.0.1:8765` only** — never `0.0.0.0`.
- **Serial** request handling: one accept loop, one request at a time (shared mutable ONNX session).
- Body size cap: **50 MB** → `413` when exceeded.
- CORS on every response: `Access-Control-Allow-Origin: *`, `Access-Control-Allow-Methods: GET, POST, OPTIONS`, `Access-Control-Allow-Headers: Content-Type, Authorization`. `OPTIONS` never requires auth.
- Optional auth: if env `DROPBG_API_TOKEN` is non-empty, `/v1/*` requires `Authorization: Bearer <token>` → `401` otherwise.
- Port-in-use must **not** crash the app: log a warning and skip the server.
- Existing Tauri commands (`remove_background`, `remove_background_batch`) keep current behavior.
- Plugin is **dev-only**: localhost goes in `devAllowedDomains`, not `allowedDomains`.
- Plugin v1 scope: **exactly one selected node with at least one `IMAGE` fill**; replace only the first IMAGE fill, preserve other fills.

---

## File Structure

- `src-tauri/Cargo.toml` — add `tiny_http` dependency.
- `src-tauri/src/api_server.rs` *(new)* — HTTP server thread, routing, CORS, auth, body-limit, handlers.
- `src-tauri/src/lib.rs` — register `mod api_server;` and spawn the server in `.setup()`.
- `src-tauri/src/commands/inference.rs` — extract `process_image_bytes`; `process_single_image` becomes a thin wrapper.
- `figma-plugin/manifest.json` *(new)* — plugin manifest with `devAllowedDomains`.
- `figma-plugin/code.js` *(new)* — main thread: selection, export, apply result.
- `figma-plugin/ui.html` *(new)* — visible iframe: status UI + `fetch`.
- `figma-plugin/README.md` *(new)* — how to load the dev plugin.

---

## Task 1: Refactor inference into a bytes-based core

Do this first so the HTTP handler (Task 3) and the existing batch command share one pipeline.

**Files:**
- Modify: `src-tauri/src/commands/inference.rs` (the `process_single_image` fn, ~line 166)
- Test: `src-tauri/src/commands/inference.rs` (inline `#[cfg(test)]` module)

**Interfaces:**
- Produces: `pub fn process_image_bytes(session_state: &SessionState, bytes: &[u8], mask_size: u32) -> Result<Vec<u8>, String>` — decodes an in-memory image, runs the model, returns PNG bytes (RGBA with alpha). Reachable as `crate::commands::process_image_bytes` via the existing `pub use inference::*`.
- Consumes: existing `crate::inference::preprocess::*`, `run_inference`, `postprocess::apply_mask_rect` (unchanged).

- [ ] **Step 1: Write the failing test**

Add at the bottom of `src-tauri/src/commands/inference.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::inference::session::SessionState;

    // A 2x2 in-memory PNG so the test needs no fixture file on disk.
    fn tiny_png() -> Vec<u8> {
        let img = image::RgbaImage::from_pixel(2, 2, image::Rgba([10, 20, 30, 255]));
        let mut buf = Vec::new();
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .unwrap();
        buf
    }

    #[test]
    fn process_image_bytes_errors_without_session() {
        // No model loaded -> the session guard holds None -> "Session not initialized".
        let state = SessionState::new();
        let err = process_image_bytes(&state, &tiny_png(), 1024).unwrap_err();
        assert!(
            err.contains("Session not initialized") || err.contains("Session lock"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn process_image_bytes_rejects_garbage() {
        let state = SessionState::new();
        let err = process_image_bytes(&state, b"not an image", 1024).unwrap_err();
        assert!(err.to_lowercase().contains("decode") || err.to_lowercase().contains("image"),
            "unexpected error: {err}");
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cd src-tauri && cargo test --lib process_image_bytes`
Expected: FAIL — `cannot find function 'process_image_bytes'`.

- [ ] **Step 3: Add `process_image_bytes` and rewrite `process_single_image` as a wrapper**

Replace the existing `process_single_image` (currently starting `fn process_single_image(session_state: &SessionState, path: &PathBuf, mask_size: u32) -> Result<Vec<u8>, String> { ... }`) with:

```rust
/// Core background-removal pipeline operating on in-memory image bytes.
/// Shared by the batch command and the localhost HTTP API.
pub fn process_image_bytes(
    session_state: &SessionState,
    bytes: &[u8],
    mask_size: u32,
) -> Result<Vec<u8>, String> {
    let img = image::load_from_memory(bytes)
        .map_err(|e| format!("Failed to decode image: {e}"))?;
    let orig_w = img.width();
    let orig_h = img.height();

    let (mask_w, mask_h) = crate::inference::preprocess::resolve_mask_size(&img, mask_size);
    let tensor = crate::inference::preprocess::preprocess(&img, mask_size)
        .map_err(|e| e.to_string())?;

    let mask_data = {
        let mut guard = session_state
            .session
            .lock()
            .map_err(|e| format!("Session lock poisoned: {e}"))?;
        let session = guard.as_mut().ok_or("Session not initialized")?;
        crate::inference::run_inference(session, tensor)?
    };

    let result_img = crate::inference::postprocess::apply_mask_rect(
        &img, &mask_data, mask_w, mask_h, orig_w, orig_h,
    )?;

    let mut buf = Vec::new();
    result_img
        .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
        .map_err(|e| format!("Failed to encode PNG: {e}"))?;
    Ok(buf)
}

fn process_single_image(
    session_state: &SessionState,
    path: &PathBuf,
    mask_size: u32,
) -> Result<Vec<u8>, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("Failed to open image: {e}"))?;
    process_image_bytes(session_state, &bytes, mask_size)
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cd src-tauri && cargo test --lib process_image_bytes`
Expected: PASS (2 tests).

- [ ] **Step 5: Confirm the full lib still builds and tests pass**

Run: `cd src-tauri && cargo test --lib`
Expected: PASS — existing tests (incl. the 10 noted in the project) still green.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/commands/inference.rs
git commit -m "refactor: extract process_image_bytes for shared bytes-based inference"
```

---

## Task 2: HTTP server skeleton (health, OPTIONS, 404, CORS, auth)

Stand up the server with everything except `/v1/remove`. Independently verifiable with `curl`.

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/api_server.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Produces: `pub fn api_server::spawn(app: tauri::AppHandle)` — spawns the server thread; returns immediately. Internal `handle_remove` is added in Task 3.
- Consumes: `crate::model::downloader::current_variant()`.

- [ ] **Step 1: Add the dependency**

In `src-tauri/Cargo.toml` under `[dependencies]`, add:

```toml
tiny_http = "0.12"
```

- [ ] **Step 2: Create `src-tauri/src/api_server.rs`**

```rust
//! Localhost HTTP API so a companion (e.g. the Figma plugin) can drive DropBG.
//! Loopback-only, serial, CORS-enabled. See docs/superpowers/specs/2026-06-30-*.

use crate::inference::session::SessionState;
use crate::model::downloader;
use std::io::Read;
use tauri::{AppHandle, Manager};
use tiny_http::{Header, Method, Response, Server};

const ADDR: &str = "127.0.0.1:8765";
const MAX_BODY: usize = 50 * 1024 * 1024; // 50 MB

/// Spawn the API server on its own thread. Never panics the app: a bind
/// failure (e.g. port in use) is logged and the server is simply not started.
pub fn spawn(app: AppHandle) {
    std::thread::Builder::new()
        .name("dropbg-api".into())
        .spawn(move || {
            let server = match Server::http(ADDR) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[api] could not bind http://{ADDR}: {e}; local API disabled");
                    return;
                }
            };
            eprintln!("[api] listening on http://{ADDR}");
            for request in server.incoming_requests() {
                handle(&app, request);
            }
        })
        .expect("failed to spawn dropbg-api thread");
}

fn cors_headers() -> Vec<Header> {
    vec![
        Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap(),
        Header::from_bytes(&b"Access-Control-Allow-Methods"[..], &b"GET, POST, OPTIONS"[..]).unwrap(),
        Header::from_bytes(&b"Access-Control-Allow-Headers"[..], &b"Content-Type, Authorization"[..]).unwrap(),
    ]
}

fn with_cors<R: Read>(mut resp: Response<R>) -> Response<R> {
    for h in cors_headers() {
        resp.add_header(h);
    }
    resp
}

fn text(code: u32, msg: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    with_cors(Response::from_string(msg).with_status_code(code)).with_header(
        Header::from_bytes(&b"Content-Type"[..], &b"text/plain; charset=utf-8"[..]).unwrap(),
    )
}

fn json(code: u32, body: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    with_cors(Response::from_string(body).with_status_code(code))
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
}

/// Returns true if the request is authorized (token unset, or matching bearer).
fn authorized(request: &tiny_http::Request) -> bool {
    let token = match std::env::var("DROPBG_API_TOKEN") {
        Ok(t) if !t.is_empty() => t,
        _ => return true, // no token configured -> open
    };
    let expected = format!("Bearer {token}");
    request
        .headers()
        .iter()
        .any(|h| h.field.equiv("Authorization") && h.value.as_str() == expected)
}

fn handle(app: &AppHandle, request: tiny_http::Request) {
    let method = request.method().clone();
    let url = request.url().to_string();

    // Preflight never requires auth.
    if method == Method::Options {
        let _ = request.respond(with_cors(Response::empty(204)));
        return;
    }

    if !authorized(&request) {
        let _ = request.respond(text(401, "Unauthorized"));
        return;
    }

    match (&method, url.as_str()) {
        (Method::Get, "/v1/health") => {
            let model = downloader::current_variant()
                .map(|v| v.name().to_string())
                .unwrap_or_else(|_| "none".into());
            let body = format!("{{\"ok\":true,\"model\":\"{model}\"}}");
            let _ = request.respond(json(200, &body));
        }
        (Method::Post, "/v1/remove") => {
            handle_remove(app, request); // implemented in Task 3
        }
        _ => {
            let _ = request.respond(text(404, "Not found"));
        }
    }
}

// --- Task 3 will add: fn handle_remove(app: &AppHandle, request: tiny_http::Request) ---
// Temporary stub so Task 2 compiles on its own:
fn handle_remove(_app: &AppHandle, request: tiny_http::Request) {
    let _ = request.respond(text(501, "Not implemented yet"));
}
```

- [ ] **Step 3: Register and spawn in `lib.rs`**

In `src-tauri/src/lib.rs`, add the module declaration near the other `mod` lines:

```rust
mod api_server;
```

And add a `.setup(...)` call to the builder chain (place it right after `.manage(HiResState::new())`):

```rust
        .setup(|app| {
            api_server::spawn(app.handle().clone());
            Ok(())
        })
```

- [ ] **Step 4: Build**

Run: `cd src-tauri && cargo check`
Expected: compiles (warning for unused `app` in the stub is fine).

- [ ] **Step 5: Manual smoke test**

In one terminal: `cd src-tauri && cargo run` (launch the app).
In another:

```bash
curl -i http://127.0.0.1:8765/v1/health
curl -i -X OPTIONS http://127.0.0.1:8765/v1/remove
curl -i http://127.0.0.1:8765/nope
```

Expected: health → `200` JSON `{"ok":true,"model":"..."}` with `Access-Control-Allow-Origin: *`; OPTIONS → `204` with CORS headers; `/nope` → `404`.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/api_server.rs src-tauri/src/lib.rs
git commit -m "feat: localhost API server skeleton (health, OPTIONS, CORS, auth)"
```

---

## Task 3: Wire `POST /v1/remove`

**Files:**
- Modify: `src-tauri/src/api_server.rs` (replace the `handle_remove` stub)

**Interfaces:**
- Consumes: `crate::commands::process_image_bytes` (Task 1), `SessionState` (managed), `downloader::current_variant().input_size()`.

- [ ] **Step 1: Replace the stub `handle_remove` with the real one**

Delete the temporary stub at the bottom of `api_server.rs` and add:

```rust
fn read_body_limited(request: &mut tiny_http::Request) -> Result<Vec<u8>, ()> {
    if let Some(len) = request.body_length() {
        if len > MAX_BODY {
            return Err(());
        }
    }
    let mut buf = Vec::new();
    // Read at most MAX_BODY + 1 so we can detect an over-limit chunked body.
    let mut reader = request.as_reader().take((MAX_BODY as u64) + 1);
    reader.read_to_end(&mut buf).map_err(|_| ())?;
    if buf.len() > MAX_BODY {
        return Err(());
    }
    Ok(buf)
}

fn handle_remove(app: &AppHandle, mut request: tiny_http::Request) {
    let bytes = match read_body_limited(&mut request) {
        Ok(b) => b,
        Err(()) => {
            let _ = request.respond(text(413, "Image too large (max 50 MB)"));
            return;
        }
    };
    if bytes.is_empty() {
        let _ = request.respond(text(400, "Empty body"));
        return;
    }

    let state = app.state::<SessionState>();
    if let Err(e) = state.ensure_loaded() {
        let _ = request.respond(text(503, &e));
        return;
    }
    let mask_size = downloader::current_variant()
        .map(|v| v.input_size())
        .unwrap_or(1024);

    match crate::commands::process_image_bytes(state.inner(), &bytes, mask_size) {
        Ok(png) => {
            let resp = with_cors(Response::from_data(png).with_status_code(200)).with_header(
                Header::from_bytes(&b"Content-Type"[..], &b"image/png"[..]).unwrap(),
            );
            let _ = request.respond(resp);
        }
        Err(e) => {
            let _ = request.respond(text(500, &e));
        }
    }
}
```

- [ ] **Step 2: Build**

Run: `cd src-tauri && cargo check`
Expected: compiles cleanly.

- [ ] **Step 3: Manual end-to-end test (requires a downloaded model)**

Launch the app (`cargo run`). With a real photo `in.png`:

```bash
# Health shows the loaded model name:
curl -s http://127.0.0.1:8765/v1/health

# Remove background -> cutout PNG with alpha:
curl -s --data-binary @in.png http://127.0.0.1:8765/v1/remove -o out.png
file out.png   # -> PNG image data, RGBA

# Garbage body -> 400:
curl -i --data-binary "garbage" http://127.0.0.1:8765/v1/remove

# Oversize guard -> 413 (make a >50MB file):
head -c 52428801 /dev/zero > big.bin
curl -i --data-binary @big.bin http://127.0.0.1:8765/v1/remove
```

Expected: valid image → `200 image/png`; garbage → `400`; oversize → `413`. If no model is downloaded, `/v1/remove` returns `503`.

- [ ] **Step 4: Token test (optional path)**

```bash
DROPBG_API_TOKEN=secret cargo run   # restart app with a token
curl -i http://127.0.0.1:8765/v1/health                                  # -> 401
curl -i -H "Authorization: Bearer secret" http://127.0.0.1:8765/v1/health # -> 200
curl -i -X OPTIONS http://127.0.0.1:8765/v1/remove                        # -> 204 (no auth needed)
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/api_server.rs
git commit -m "feat: POST /v1/remove with 50MB guard and error mapping"
```

---

## Task 4: Figma plugin shell (manifest + visible UI + health check)

**Files:**
- Create: `figma-plugin/manifest.json`
- Create: `figma-plugin/code.js`
- Create: `figma-plugin/ui.html`
- Create: `figma-plugin/README.md`

**Interfaces:**
- The plugin UI calls `GET <base>/v1/health` where `<base>` is the first of `http://127.0.0.1:8765`, `http://localhost:8765` that responds.
- postMessage contract (used here and in Tasks 5–6):
  - plugin→UI: `{ type: "health-check" }`, `{ type: "remove", requestId, bytes }`
  - UI→plugin: `{ type: "health-ok", model }`, `{ type: "health-error", message }`, `{ type: "remove-ok", requestId, bytes }`, `{ type: "remove-error", requestId, status?, message }`

- [ ] **Step 1: Create `figma-plugin/manifest.json`**

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

- [ ] **Step 2: Create `figma-plugin/code.js` (shell + message routing)**

```js
// Main thread. Has the Figma API; cannot fetch.
figma.showUI(__html__, { width: 320, height: 160 });

// Route messages from the UI iframe.
figma.ui.onmessage = (msg) => {
  if (msg.type === "health-ok") {
    figma.notify(`DropBG connected (model: ${msg.model})`);
  } else if (msg.type === "health-error") {
    figma.notify(`DropBG not reachable: ${msg.message}`);
  }
  // "remove-ok" / "remove-error" handled in Task 5–6.
};

// Kick off a health check on open.
figma.ui.postMessage({ type: "health-check" });
```

- [ ] **Step 3: Create `figma-plugin/ui.html` (visible status + fetch)**

```html
<!DOCTYPE html>
<html>
  <body style="font:13px -apple-system,sans-serif;margin:12px;color:#333">
    <div style="font-weight:600;margin-bottom:8px">DropBG Local</div>
    <div id="status">Checking…</div>
    <button id="run" style="margin-top:12px" disabled>Remove background</button>

    <script>
      const BASES = ["http://127.0.0.1:8765", "http://localhost:8765"];
      let base = null;
      const statusEl = document.getElementById("status");
      const runBtn = document.getElementById("run");

      async function resolveBase() {
        for (const b of BASES) {
          try {
            const r = await fetch(b + "/v1/health");
            if (r.ok) {
              const j = await r.json();
              base = b;
              return j.model || "unknown";
            }
          } catch (_) { /* try next */ }
        }
        return null;
      }

      async function health() {
        const model = await resolveBase();
        if (model) {
          statusEl.textContent = "Connected (model: " + model + ")";
          runBtn.disabled = false;
          parent.postMessage({ pluginMessage: { type: "health-ok", model } }, "*");
        } else {
          statusEl.textContent = "Not connected. Start DropBG and reopen.";
          runBtn.disabled = true;
          parent.postMessage({ pluginMessage: { type: "health-error", message: "no server on 8765" } }, "*");
        }
      }

      // Receive messages from the main thread.
      onmessage = (e) => {
        const msg = e.data.pluginMessage;
        if (!msg) return;
        if (msg.type === "health-check") health();
        // "remove" handled in Task 5.
      };

      runBtn.onclick = () => {
        parent.postMessage({ pluginMessage: { type: "run-clicked" } }, "*");
      };
    </script>
  </body>
</html>
```

- [ ] **Step 4: Create `figma-plugin/README.md`**

```markdown
# DropBG Local (Figma dev plugin)

Thin companion that sends the selected image to a locally running DropBG app
and replaces it with the background removed. No cloud upload.

## Use
1. Launch the DropBG desktop app (its localhost API listens on `127.0.0.1:8765`).
2. In Figma desktop: **Plugins → Development → Import plugin from manifest…**
   and pick `figma-plugin/manifest.json`.
3. Select one image layer, run **DropBG Local**, click **Remove background**.

The localhost host is declared under `devAllowedDomains`, so this works as a
development plugin without publishing. To publish, move the host to
`allowedDomains` and add a `reasoning` string (Figma requires it).
```

- [ ] **Step 5: Manual test (shell)**

Launch DropBG. In Figma desktop → Plugins → Development → Import plugin from manifest → select `figma-plugin/manifest.json`. Run the plugin.
Expected: a 320×160 panel showing "Connected (model: …)" and an enabled button; a toast "DropBG connected (model: …)". Quit DropBG, reopen the plugin → "Not connected…" and disabled button.

- [ ] **Step 6: Commit**

```bash
git add figma-plugin/
git commit -m "feat: Figma plugin shell with localhost health check"
```

---

## Task 5: Selection export + remove round-trip

**Files:**
- Modify: `figma-plugin/code.js`
- Modify: `figma-plugin/ui.html`

**Interfaces:**
- Consumes the message contract from Task 4. Adds `requestId` correlation.
- Produces (for Task 6): on `remove-ok`, `code.js` has `msg.bytes` (Uint8Array PNG) ready to apply.

- [ ] **Step 1: Add selection validation + export in `code.js`**

Replace the body of `figma.ui.onmessage` and add a counter. Full new `code.js`:

```js
figma.showUI(__html__, { width: 320, height: 160 });

let reqCounter = 0;

function firstImageFillIndex(node) {
  if (!("fills" in node) || !Array.isArray(node.fills)) return -1;
  return node.fills.findIndex((f) => f.type === "IMAGE");
}

async function startRemoval() {
  const sel = figma.currentPage.selection;
  if (sel.length === 0) { figma.notify("Select an image layer first."); return; }
  if (sel.length > 1) { figma.notify("Select exactly one image layer."); return; }
  const node = sel[0];
  if (firstImageFillIndex(node) === -1) {
    figma.notify("This layer has no image fill to process.");
    return;
  }
  const bytes = await node.exportAsync({ format: "PNG" });
  const requestId = `r${++reqCounter}`;
  figma.ui.postMessage({ type: "remove", requestId, bytes });
}

figma.ui.onmessage = (msg) => {
  if (msg.type === "health-ok") {
    figma.notify(`DropBG connected (model: ${msg.model})`);
  } else if (msg.type === "health-error") {
    figma.notify(`DropBG not reachable: ${msg.message}`);
  } else if (msg.type === "run-clicked") {
    startRemoval();
  } else if (msg.type === "remove-error") {
    const code = msg.status ? ` (${msg.status})` : "";
    figma.notify(`Background removal failed${code}: ${msg.message}`);
  } else if (msg.type === "remove-ok") {
    // Task 6 applies msg.bytes.
    figma.notify("Cutout received.");
  }
};

figma.ui.postMessage({ type: "health-check" });
```

- [ ] **Step 2: Add the POST handler in `ui.html`**

Extend the `onmessage` handler in `ui.html` to handle `remove`:

```js
      onmessage = (e) => {
        const msg = e.data.pluginMessage;
        if (!msg) return;
        if (msg.type === "health-check") health();
        if (msg.type === "remove") doRemove(msg);
      };

      async function doRemove(msg) {
        if (!base) {
          parent.postMessage({ pluginMessage: { type: "remove-error", requestId: msg.requestId, message: "Not connected" } }, "*");
          return;
        }
        statusEl.textContent = "Removing…";
        try {
          const r = await fetch(base + "/v1/remove", { method: "POST", body: msg.bytes });
          if (!r.ok) {
            const errText = await r.text();
            parent.postMessage({ pluginMessage: { type: "remove-error", requestId: msg.requestId, status: r.status, message: errText } }, "*");
            statusEl.textContent = "Failed.";
            return;
          }
          const buf = new Uint8Array(await r.arrayBuffer());
          parent.postMessage({ pluginMessage: { type: "remove-ok", requestId: msg.requestId, bytes: buf } }, "*");
          statusEl.textContent = "Done.";
        } catch (err) {
          parent.postMessage({ pluginMessage: { type: "remove-error", requestId: msg.requestId, message: "Is DropBG running?" } }, "*");
          statusEl.textContent = "Not connected.";
        }
      }
```

- [ ] **Step 3: Manual test (round-trip, no apply yet)**

Reload the plugin (Plugins → Development → DropBG Local). Select an image layer, click **Remove background**.
Expected: status goes "Removing…" → "Done." and a toast "Cutout received." Select nothing / two layers / a text layer → the matching validation toast, no request sent. Quit DropBG and retry → "Is DropBG running?" toast.

- [ ] **Step 4: Commit**

```bash
git add figma-plugin/code.js figma-plugin/ui.html
git commit -m "feat: export selection and round-trip cutout through localhost API"
```

---

## Task 6: Apply result — replace first IMAGE fill, keep original hidden

**Files:**
- Modify: `figma-plugin/code.js` (the `remove-ok` branch)

**Interfaces:**
- Consumes: `remove-ok` `msg.bytes` (Uint8Array PNG) from Task 5.

- [ ] **Step 1: Track the active node and apply the cutout**

In `code.js`, store the node when starting removal, and apply on success. Change `startRemoval` to remember the node, and replace the `remove-ok` branch:

In `startRemoval`, after validation add:

```js
  activeNode = node;
```

Add a module-level `let activeNode = null;` near `let reqCounter = 0;`.

Replace the `remove-ok` branch with:

```js
  } else if (msg.type === "remove-ok") {
    applyCutout(msg.bytes);
  }
```

Add the apply function:

```js
function applyCutout(bytes) {
  const node = activeNode;
  if (!node) { figma.notify("No target layer."); return; }
  const idx = firstImageFillIndex(node);
  if (idx === -1) { figma.notify("Target layer lost its image fill."); return; }

  // Hidden backup of the original (keeps its fills).
  const backup = node.clone();
  backup.visible = false;
  backup.name = `${node.name} (original)`;

  // Replace only the first IMAGE fill; preserve all other fills.
  const image = figma.createImage(bytes);
  const fills = JSON.parse(JSON.stringify(node.fills)); // clone the readonly array
  fills[idx] = { type: "IMAGE", scaleMode: "FILL", imageHash: image.hash };
  node.fills = fills;

  figma.currentPage.selection = [node];
  figma.notify("Background removed.");
}
```

- [ ] **Step 2: Manual test (full happy path)**

Reload the plugin. Select a rectangle with an image fill → **Remove background**.
Expected: the fill becomes the cut-out image in place; a sibling layer `"<name> (original)"` exists and is hidden; the processed node is selected; `Cmd+Z` once reverts the change. Repeat with a node that has an image fill **plus** a solid overlay fill → the solid fill is preserved (only the image fill swaps).

- [ ] **Step 3: Commit**

```bash
git add figma-plugin/code.js
git commit -m "feat: replace first IMAGE fill in place, keep hidden original backup"
```

---

## Self-Review Notes

- **Spec coverage:** health + remove endpoints (Tasks 2–3), CORS/auth/serial/413/503 (Tasks 2–3), bytes-based engine reuse (Task 1), `devAllowedDomains` manifest (Task 4), visible UI + base fallback (Task 4), message contract (Tasks 4–5), IMAGE-fill-only scope + first-fill replacement + hidden backup (Tasks 5–6). All present.
- **Out of v1 (not in this plan, per spec):** background-replace/upscale/auto-crop endpoints, auto-routing in HTTP path, multi-node support, Community publishing, configurable port.
- **Type consistency:** `process_image_bytes(&SessionState, &[u8], u32) -> Result<Vec<u8>, String>` defined in Task 1, called identically in Task 3. `firstImageFillIndex` / `activeNode` defined in Task 5 and reused in Task 6. Message `type` strings match across `code.js` and `ui.html`.
- **Manual-test rationale:** the server's HTTP surface and the Figma plugin have no cheap unit-test harness here; Task 1's pure function carries the automated coverage, the rest is `curl` + in-Figma verification with explicit expected results.
