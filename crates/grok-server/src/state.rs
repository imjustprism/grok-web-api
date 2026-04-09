use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use grok_client::GrokClient;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub client: Arc<GrokClient>,
    pub config: Arc<Config>,
    pub started_at: Instant,
    pub request_count: Arc<AtomicU64>,
    pub last_request_at: Arc<AtomicU64>,
}

impl AppState {
    #[must_use]
    pub fn new(client: GrokClient, config: Config) -> Self {
        Self {
            client: Arc::new(client),
            config: Arc::new(config),
            started_at: Instant::now(),
            request_count: Arc::new(AtomicU64::new(0)),
            last_request_at: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn record_request(&self) {
        self.request_count.fetch_add(1, Ordering::Relaxed);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.last_request_at.store(now, Ordering::Relaxed);
    }
}
