use crate::model::downloader::CloudProvider;
use serde::Serialize;
use std::sync::{Arc, Mutex};

/// Estimated cost per image for each provider (USD).
fn cost_per_image(provider: &CloudProvider) -> f64 {
    match provider {
        CloudProvider::Replicate => 0.0004,
        CloudProvider::FalAI => 0.018,
        CloudProvider::RemoveBg => 0.10,
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderUsage {
    pub provider: String,
    pub provider_name: String,
    pub image_count: u32,
    pub estimated_cost: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CloudUsageSummary {
    pub total_images: u32,
    pub total_estimated_cost: f64,
    pub by_provider: Vec<ProviderUsage>,
}

/// Session-scoped, in-memory cloud API usage tracker.
#[derive(Clone)]
pub struct CloudUsageState {
    inner: Arc<Mutex<UsageInner>>,
}

#[derive(Default)]
struct UsageInner {
    replicate: u32,
    fal_ai: u32,
    remove_bg: u32,
}

impl CloudUsageState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(UsageInner::default())),
        }
    }

    /// Record one successful cloud API call.
    pub fn record(&self, provider: &CloudProvider) {
        if let Ok(mut inner) = self.inner.lock() {
            match provider {
                CloudProvider::Replicate => inner.replicate += 1,
                CloudProvider::FalAI => inner.fal_ai += 1,
                CloudProvider::RemoveBg => inner.remove_bg += 1,
            }
        }
    }

    /// Get usage summary for the current session.
    pub fn summary(&self) -> CloudUsageSummary {
        let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());

        let mut by_provider = Vec::new();
        let mut total_images = 0u32;
        let mut total_cost = 0.0f64;

        for (provider, count) in [
            (CloudProvider::Replicate, inner.replicate),
            (CloudProvider::FalAI, inner.fal_ai),
            (CloudProvider::RemoveBg, inner.remove_bg),
        ] {
            if count > 0 {
                let cost = count as f64 * cost_per_image(&provider);
                by_provider.push(ProviderUsage {
                    provider: provider.variant_key().to_string(),
                    provider_name: provider.name().to_string(),
                    image_count: count,
                    estimated_cost: cost,
                });
                total_images += count;
                total_cost += cost;
            }
        }

        CloudUsageSummary {
            total_images,
            total_estimated_cost: total_cost,
            by_provider,
        }
    }

    /// Reset all counters.
    pub fn reset(&self) {
        if let Ok(mut inner) = self.inner.lock() {
            *inner = UsageInner::default();
        }
    }
}
