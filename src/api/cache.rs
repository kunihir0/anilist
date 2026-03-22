use moka::future::Cache as MokaCache;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

// ─── High-Performance In-Memory Cache ─────────────────────────────────────────

#[derive(Clone)]
struct CacheEntry {
    value: serde_json::Value,
    expires_at: Instant,
}

#[derive(Clone)]
pub struct Cache {
    /// The underlying moka cache. Handles concurrent access and capacity bounds.
    inner: MokaCache<String, CacheEntry>,
    /// Default time-to-live for cache entries.
    ttl: Duration,
}

impl Cache {
    /// Creates a new Cache with a default TTL and a maximum capacity.
    /// Max capacity is hardcoded to 10,000 items to prevent unbounded memory growth.
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            inner: MokaCache::builder()
                .max_capacity(10_000)
                .time_to_idle(Duration::from_secs(ttl_seconds * 2)) // Evict if unaccessed
                .build(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    /// Retrieves an item from the cache, verifying its custom TTL.
    pub async fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        if let Some(entry) = self.inner.get(key).await {
            if entry.expires_at > Instant::now() {
                return serde_json::from_value(entry.value).ok();
            } else {
                // Background eviction will eventually clean this up,
                // but we proactively invalidate it here.
                self.inner.invalidate(key).await;
            }
        }
        None
    }

    /// Sets an item in the cache with the default TTL.
    pub async fn set<T: serde::Serialize>(&self, key: String, value: &T) {
        if let Ok(val) = serde_json::to_value(value) {
            let entry = CacheEntry {
                value: val,
                expires_at: Instant::now() + self.ttl,
            };
            self.inner.insert(key, entry).await;
        }
    }

    /// Sets an item in the cache with a custom TTL.
    pub async fn set_with_ttl<T: serde::Serialize>(
        &self,
        key: String,
        value: &T,
        ttl_seconds: u64,
    ) {
        if let Ok(val) = serde_json::to_value(value) {
            let entry = CacheEntry {
                value: val,
                expires_at: Instant::now() + Duration::from_secs(ttl_seconds),
            };
            self.inner.insert(key, entry).await;
        }
    }

    /// Returns the approximate number of entries currently in the cache.
    pub fn entry_count(&self) -> u64 {
        self.inner.entry_count()
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

    pub async fn check(&self) -> Result<(), crate::models::bot_data::BotError> {
        let mut inner = self.inner.write().await;
        let now = Instant::now();
        let elapsed = now.duration_since(inner.last_refill).as_secs_f64();

        inner.tokens = (inner.tokens + elapsed * inner.refill_rate).min(inner.max_tokens);
        inner.last_refill = now;

        if inner.tokens >= 1.0 {
            inner.tokens -= 1.0;
            Ok(())
        } else {
            Err(crate::models::bot_data::BotError::Internal(
                "AniList rate limit reached. Please try again in a few seconds.".into(),
            ))
        }
    }
}
