use crate::model::downloader;
use ort::session::Session;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct SessionState {
    pub session: Arc<Mutex<Option<Session>>>,
}

// Safety: We protect Session with a Mutex so only one thread accesses it at a time.
unsafe impl Send for SessionState {}
unsafe impl Sync for SessionState {}

impl SessionState {
    pub fn new() -> Self {
        Self {
            session: Arc::new(Mutex::new(None)),
        }
    }

    pub fn ensure_loaded(&self) -> Result<(), String> {
        let mut guard = self.session.lock().unwrap();
        if guard.is_none() {
            let model_path = downloader::model_path().map_err(|e| e.to_string())?;
            if !model_path.exists() {
                return Err("Model not downloaded yet. Please download the model first.".into());
            }

            let session = Session::builder()
                .map_err(|e| format!("Failed to create session builder: {e}"))?
                .with_execution_providers([
                    ort::execution_providers::CoreMLExecutionProvider::default().build(),
                ])
                .map_err(|e| format!("Failed to set execution provider: {e}"))?
                .commit_from_file(&model_path)
                .map_err(|e| format!("Failed to load model: {e}"))?;

            *guard = Some(session);
        }
        Ok(())
    }

    /// Clear the loaded session so the next `ensure_loaded` will reload from disk.
    /// Used when switching model variants.
    pub fn clear(&self) {
        let mut guard = self.session.lock().unwrap();
        *guard = None;
    }
}
