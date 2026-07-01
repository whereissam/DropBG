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

    // Auth is scoped to the versioned API surface; unknown routes must still
    // 404 even when a token is configured.
    if url.starts_with("/v1/") && !authorized(&request) {
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
