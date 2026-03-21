use reqwest::Client;
use crate::api::cache::{Cache, RateLimiter};
use crate::store::Store;
use std::sync::Arc;
use tokio_cron_scheduler::JobScheduler;

// ─── Shared application state ─────────────────────────────────────────────────

#[derive(Clone)]
pub struct Data {
    pub http_client: Client,
    /// Response cache — keyed by "type:search_term", TTL 5 minutes.
    pub cache: Cache,
    /// Global rate limiter — AniList enforces 90 req/min per IP.
    pub rate_limiter: RateLimiter,
    /// Persistence store.
    pub store: Arc<Store>,
    /// Live job scheduler.
    pub scheduler: JobScheduler,
}

// ─── Shared Types ─────────────────────────────────────────────────────────────

#[derive(Debug, poise::ChoiceParameter, Clone, Copy, PartialEq, Eq)]
pub enum MediaType {
    #[name = "ANIME"]
    Anime,
    #[name = "MANGA"]
    Manga,
}

impl MediaType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MediaType::Anime => "ANIME",
            MediaType::Manga => "MANGA",
        }
    }
}

// ─── Poise type aliases ───────────────────────────────────────────────────────

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;