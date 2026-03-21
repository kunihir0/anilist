use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use std::sync::Arc;

// ─── Simple In-Memory Cache ───────────────────────────────────────────────────

struct CacheEntry {
    value: serde_json::Value,
    expires_at: Instant,
}

#[derive(Clone)]
pub struct Cache {
    /// Shared state across all clones.
    inner: Arc<RwLock<HashMap<String, CacheEntry>>>,
    /// Default time-to-live.
    ttl: Duration,
}

impl Cache {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    pub async fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        let mut inner = self.inner.write().await;
        
        if let Some(entry) = inner.get(key) {
            if entry.expires_at > Instant::now() {
                return serde_json::from_value(entry.value.clone()).ok();
            }
            // Evict expired entry.
            inner.remove(key);
        }
        None
    }

    pub async fn set<T: serde::Serialize>(&self, key: String, value: &T) {
        if let Ok(val) = serde_json::to_value(value) {
            let mut inner = self.inner.write().await;
            inner.insert(key, CacheEntry {
                value: val,
                expires_at: Instant::now() + self.ttl,
            });
        }
    }

    pub async fn set_with_ttl<T: serde::Serialize>(&self, key: String, value: &T, ttl_seconds: u64) {
        if let Ok(val) = serde_json::to_value(value) {
            let mut inner = self.inner.write().await;
            inner.insert(key, CacheEntry {
                value: val,
                expires_at: Instant::now() + Duration::from_secs(ttl_seconds),
            });
        }
    }
}

// ─── Basic Token Bucket Rate Limiter ──────────────────────────────────────────

#[derive(Clone)]
pub struct RateLimiter {
    inner: Arc<RwLock<RateLimiterInner>>,
}

struct RateLimiterInner {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64,
    last_refill: Instant,
}

impl RateLimiter {
    pub fn new(requests_per_window: u64, window_seconds: u64) -> Self {
        let refill_rate = requests_per_window as f64 / window_seconds as f64;
        Self {
            inner: Arc::new(RwLock::new(RateLimiterInner {
                tokens: requests_per_window as f64,
                max_tokens: requests_per_window as f64,
                refill_rate,
                last_refill: Instant::now(),
            })),
        }
    }

    pub async fn check(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut inner = self.inner.write().await;
        let now = Instant::now();
        let elapsed = now.duration_since(inner.last_refill).as_secs_f64();
        
        inner.tokens = (inner.tokens + elapsed * inner.refill_rate).min(inner.max_tokens);
        inner.last_refill = now;

        if inner.tokens >= 1.0 {
            inner.tokens -= 1.0;
            Ok(())
        } else {
            Err("AniList rate limit reached. Please try again in a few seconds.".into())
        }
    }
}
