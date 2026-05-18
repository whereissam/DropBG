//! Tauri command surface, split by domain.
//!
//! Each `#[tauri::command]` lives in a submodule and is re-exported here so
//! `lib.rs`'s `tauri::generate_handler![commands::*]` references keep working
//! without changes.

use serde::Serialize;
use tauri::{AppHandle, Emitter};

mod cloud;
mod editing;
mod inference;
mod model;
mod postprocess;
mod system;

pub use cloud::*;
pub use editing::*;
pub use inference::*;
pub use model::*;
pub use postprocess::*;
pub use system::*;

// ===== Shared progress events =====

#[derive(Clone, Serialize)]
pub(crate) struct ProcessProgress {
    pub step: String,
    pub percent: f64,
}

#[derive(Clone, Serialize)]
pub(crate) struct BatchProgress {
    pub index: usize,
    pub total: usize,
    pub filename: String,
    pub status: String, // "processing" | "done" | "error"
    pub error: Option<String>,
    pub output_path: Option<String>,
}

pub(crate) fn emit_progress(app: &AppHandle, step: &str, percent: f64) {
    let _ = app.emit(
        "process-progress",
        ProcessProgress {
            step: step.to_string(),
            percent,
        },
    );
}
