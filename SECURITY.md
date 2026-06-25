# Security

DropBG is a local‑first macOS desktop app (Tauri v2 — a Rust backend with a
React/WebView frontend) for removing image backgrounds. Image processing runs
**on‑device** by default; an optional "Cloud API" mode uploads images to a
third‑party provider you configure with your own API key.

This document describes the security model, the issues found during a code
review, the fixes applied, and the hardening still recommended.

---

## Security model

**Trust boundaries**

| Boundary | Notes |
| --- | --- |
| WebView frontend → Rust backend | All privileged work (file I/O, network, process spawn) happens in Rust `#[tauri::command]` handlers. The frontend is treated as the lower‑trust tier. |
| App → model hosts (HuggingFace) | ONNX models are downloaded over HTTPS and then executed by the ONNX Runtime. |
| App → cloud providers | In Cloud mode, image bytes + your API key are sent to Replicate / fal.ai / remove.bg / Photoroom. |
| App → local disk | Reads dropped images; writes PNG output to a user‑chosen folder; stores config (incl. API keys) under the app data directory. |

**Assets worth protecting**

- Cloud provider **API keys** stored on disk.
- The user's **images** (privacy — they leave the machine only in Cloud mode).
- **Integrity of model binaries** loaded into the inference runtime.
- The **command surface** exposed to the WebView (file read/write, browser/Finder launch).

---

## Findings & fixes

The following issues were identified and **fixed** in this review.

### 1. Content Security Policy was disabled — *Medium/High*

`src-tauri/tauri.conf.json` had `app.security.csp = null`, which disables CSP
entirely. The frontend renders attacker‑influenced strings (image filenames,
error messages, file paths) and has access to powerful Tauri commands, so any
injected markup/script could pivot into reading/writing files or launching
processes.

**Fix:** a restrictive CSP is now set:

```
default-src 'self'; img-src 'self' data: blob:; style-src 'self' 'unsafe-inline';
script-src 'self'; connect-src 'self' ipc: http://ipc.localhost; font-src 'self' data:;
object-src 'none'; base-uri 'self'; form-action 'none'; frame-ancestors 'none'
```

`img-src data: blob:` is required because previews use base64 data URIs and
object URLs; `style-src 'unsafe-inline'` is required for React inline styles.
No remote origins are permitted.

### 2. Over‑broad filesystem capability — *Medium*

`src-tauri/capabilities/default.json` granted the WebView `fs:allow-write` in
addition to `fs:allow-read`, unscoped. The frontend only ever calls `readFile`
(to preview a dropped image); all writes happen in Rust commands, which are not
governed by the plugin ACL.

**Fix:** removed `fs:allow-write` from the capability. The WebView keeps read
access (needed for previews) but can no longer write arbitrary files via the
`fs` plugin. This shrinks the blast radius of any frontend compromise.

### 3. `open_url_in_browser` allowlist too coarse — *Low/Medium (also a bug)*

The command only allowed `https://huggingface.co/` via a `starts_with` prefix
check. That simultaneously (a) **broke** the in‑app provider "View pricing"
links (replicate.com, fal.ai, remove.bg, photoroom.com) and (b) used a brittle
prefix test rather than a real host check.

**Fix:** the URL is now parsed to extract its host and validated against an
explicit allowlist (`huggingface.co`, `replicate.com`, `fal.ai`, `remove.bg`,
`photoroom.com`) with exact‑host or proper subdomain (`.host`) matching. It
requires `https://`, rejects control characters/whitespace and embedded
credentials (`user@host`). The URL is passed to `open` as a single argument
(no shell), and the `https://` prefix guarantees it can't be misread as a flag.

### 4. Plaintext API keys on disk — *Medium*

Cloud API keys are stored in `~/Library/Application Support/com.dropbg.app/config.json`
in plaintext. The file was created with default permissions, leaving the keys
readable by other local accounts/processes depending on the parent directory's
mode.

**Fix:** `config.json` is now written with `0600` (owner read/write only) on
Unix. See "Remaining recommendations" for the stronger keychain‑based fix.

### 5. Model downloads not constrained to TLS — *Low (defense‑in‑depth)*

Downloaded ONNX models are executed by the ONNX Runtime, so their integrity
matters. URLs are hardcoded HTTPS today, but the download path did not enforce
this and gave a confusing failure for the manual‑export‑only variants.

**Fix:** the downloader now rejects empty URLs with a clear "manual download"
message and refuses any non‑`https://` URL.

---

## Items reviewed and considered acceptable

- **Command injection:** `open` / `open -R` are invoked via
  `std::process::Command` with arguments (no shell), so paths/URLs are not
  interpreted by a shell. `open_path_in_finder` also rejects NUL bytes.
- **`save_image`:** restricted to `.png` output; data comes from a save dialog.
- **Arbitrary read/write paths** (`set_model_dir`, `set_output_dir`,
  `replace_background_image` background path): these target the *user's own*
  files via OS file/folder pickers on a single‑user desktop app, so they are
  within the user's own authority, not a privilege boundary.
- **Cloud result download (`download_replicate_output`, fal.ai image URL):** the
  app fetches a result image from a URL returned by the provider's API. This is
  a mild SSRF‑shaped pattern but the providers are trusted and the response is
  re‑decoded as an image before use. Tracked below as a hardening item.
- **No secrets in the repository or git history** (scanned): API keys are only
  ever entered at runtime.

---

## Remaining recommendations (not yet implemented)

1. **Store API keys in the macOS Keychain** instead of `config.json`. The
   `0600` permission helps, but the Keychain provides OS‑level access control
   and is the correct home for credentials. (`security-framework` /
   `keyring` crate.)
2. **Pin model checksums.** Record an expected SHA‑256 per model variant and
   verify it after download (and before loading into ORT) to defend against a
   compromised/redirected upstream host. Not added here because it requires
   capturing trusted hashes for every variant first.
3. **Bound download size.** Add a maximum byte cap while streaming downloads to
   avoid filling the disk if a host misbehaves.
4. **Validate cloud result URLs** (scheme = https, host belongs to the selected
   provider) before fetching the result image.
5. **Keep the CSP tight.** If a future feature needs a remote origin, add the
   specific host rather than relaxing `default-src`.

---

## Reporting a vulnerability

Please report security issues privately to the maintainer rather than opening a
public issue. Include reproduction steps and the affected version. We aim to
acknowledge reports within a few days.
