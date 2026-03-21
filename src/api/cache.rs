use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use serde::{Serialize, de::DeserializeOwned};

// ─── Response Cache ───────────────────────────────────────────────────────────
//
// Stores serialised AniList responses keyed by "type:query" (e.g. "anime:Naruto").
// Entries expire after `ttl`. All cached types must implement Serialize +
// DeserializeOwned — enforced at the call site, not here.

pub struct Cache {
    store: Mutex<HashMap<String, (serde_json::Value, Instant)>>,
    ttl: Duration,
}

impl Cache {
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            store: Mutex::new(HashMap::new()),
            ttl: Duration::from_secs(ttl_secs),
        }
    }

    /// Returns a cached value if it exists and has not expired.
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let store = self.store.lock().await;
        if let Some((value, inserted_at)) = store.get(key) {
            if inserted_at.elapsed() < self.ttl {
                return serde_json::from_value(value.clone()).ok();
            }
        }
        None
    }

    /// Serialises and stores a value under `key`.
    pub async fn set<T: Serialize>(&self, key: String, value: &T) {
        if let Ok(json) = serde_json::to_value(value) {
            let mut store = self.store.lock().await;
            store.insert(key, (json, Instant::now()));
        }
    }
}

// ─── Rate Limiter ─────────────────────────────────────────────────────────────
//
// Tracks outgoing API calls in a sliding window. AniList allows 90 req/min.
// Rather than queuing, we return a descriptive error so the user gets immediate
// feedback instead of a silent hang.

pub struct RateLimiter {
    state: Mutex<(u32, Instant)>,
    limit: u32,
    window: Duration,
}

impl RateLimiter {
    pub fn new(limit: u32, window_secs: u64) -> Self {
        Self {
            state: Mutex::new((0, Instant::now())),
            limit,
            window: Duration::from_secs(window_secs),
        }
    }

    /// Returns `Ok(())` if a request can proceed, or an error string if the
    /// rate limit has been reached for the current window.
    pub async fn check(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut guard = self.state.lock().await;
        let (count, window_start) = &mut *guard;

        if window_start.elapsed() >= self.window {
            *count = 0;
            *window_start = Instant::now();
        }

        if *count >= self.limit {
            let remaining = self.window.saturating_sub(window_start.elapsed());
            return Err(format!(
                "Rate limit reached ({} requests/min). Try again in {}s.",
                self.limit,
                remaining.as_secs()
            )
            .into());
        }

        *count += 1;
        Ok(())
    }
}