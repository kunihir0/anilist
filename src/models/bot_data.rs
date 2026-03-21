use reqwest::Client;
use crate::api::cache::{Cache, RateLimiter};

// ─── Shared application state ─────────────────────────────────────────────────

pub struct Data {
    pub http_client: Client,
    /// Response cache — keyed by "type:search_term", TTL 5 minutes.
    pub cache: Cache,
    /// Global rate limiter — AniList enforces 90 req/min per IP.
    pub rate_limiter: RateLimiter,
}

// ─── Poise type aliases ───────────────────────────────────────────────────────

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;