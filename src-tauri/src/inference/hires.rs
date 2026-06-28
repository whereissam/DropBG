//! Caches the most recent local cutout's high-precision (f32) alpha (Phase 11.5
//! follow-up). The interactive flow round-trips the result through the frontend
//! as an 8-bit PNG, so by the time a 16-bit export is requested the f32 alpha
//! the model produced is gone. Stashing it here lets the 16-bit export use true
//! alpha precision instead of promoting the 8-bit channel (`a8 * 257`).
//!
//! Safety model: the cache is only trusted when the base64 handed to the export
//! command is byte-identical to the preview this alpha was produced for. If the
//! user edited the result (auto-crop, background replace, HR refine, …) the
//! base64 differs and we fall back to the 8-bit-derived alpha — the 16-bit
//! container is still written, just without sub-8-bit alpha for that case.

use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct HiResCutout {
    /// The exact base64 PNG returned to the frontend for this cutout.
    pub preview_b64: String,
    pub width: u32,
    pub height: u32,
    /// Full-resolution alpha, 0..1, length `width * height`.
    pub alpha: Vec<f32>,
}

#[derive(Clone, Default)]
pub struct HiResState {
    inner: Arc<Mutex<Option<HiResCutout>>>,
}

impl HiResState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn store(&self, cutout: HiResCutout) {
        if let Ok(mut g) = self.inner.lock() {
            *g = Some(cutout);
        }
    }

    /// The cached f32 alpha, but only if it was produced for exactly this preview
    /// base64 (unmodified) at these dimensions.
    pub fn alpha_for(&self, preview_b64: &str, width: u32, height: u32) -> Option<Vec<f32>> {
        let g = self.inner.lock().ok()?;
        let c = g.as_ref()?;
        if c.preview_b64 == preview_b64 && c.width == width && c.height == height {
            Some(c.alpha.clone())
        } else {
            None
        }
    }
}
