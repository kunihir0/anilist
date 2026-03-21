use reqwest::Client;
use crate::api::cache::{Cache, RateLimiter};
use crate::store::Store;
use std::sync::Arc;
use tokio_cron_scheduler::JobScheduler;
use tokio::sync::RwLock;

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
    /// Cached genres for autocomplete.
    pub genres: Arc<RwLock<Vec<String>>>,
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

#[derive(Debug, poise::ChoiceParameter, Clone, Copy, PartialEq, Eq)]
pub enum MediaFormat {
    #[name = "TV"]
    Tv,
    #[name = "TV Short"]
    TvShort,
    #[name = "Movie"]
    Movie,
    #[name = "Special"]
    Special,
    #[name = "OVA"]
    Ova,
    #[name = "ONA"]
    Ona,
    #[name = "Music"]
    Music,
    #[name = "Manga"]
    Manga,
    #[name = "One Shot"]
    OneShot,
    #[name = "Novel"]
    Novel,
}

impl MediaFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            MediaFormat::Tv => "TV",
            MediaFormat::TvShort => "TV_SHORT",
            MediaFormat::Movie => "MOVIE",
            MediaFormat::Special => "SPECIAL",
            MediaFormat::Ova => "OVA",
            MediaFormat::Ona => "ONA",
            MediaFormat::Music => "MUSIC",
            MediaFormat::Manga => "MANGA",
            MediaFormat::OneShot => "ONE_SHOT",
            MediaFormat::Novel => "NOVEL",
        }
    }
}

#[derive(Debug, poise::ChoiceParameter, Clone, Copy, PartialEq, Eq)]
pub enum MediaStatus {
    #[name = "Finished"]
    Finished,
    #[name = "Releasing"]
    Releasing,
    #[name = "Not Yet Released"]
    NotYetReleased,
    #[name = "Cancelled"]
    Cancelled,
    #[name = "Hiatus"]
    Hiatus,
}

impl MediaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            MediaStatus::Finished => "FINISHED",
            MediaStatus::Releasing => "RELEASING",
            MediaStatus::NotYetReleased => "NOT_YET_RELEASED",
            MediaStatus::Cancelled => "CANCELLED",
            MediaStatus::Hiatus => "HIATUS",
        }
    }
}

#[derive(Debug, poise::ChoiceParameter, Clone, Copy, PartialEq, Eq)]
pub enum MediaSort {
    #[name = "Popularity"]
    PopularityDesc,
    #[name = "Score"]
    ScoreDesc,
    #[name = "Trending"]
    TrendingDesc,
    #[name = "Latest"]
    IdDesc,
}

impl MediaSort {
    pub fn as_str(&self) -> &'static str {
        match self {
            MediaSort::PopularityDesc => "POPULARITY_DESC",
            MediaSort::ScoreDesc => "SCORE_DESC",
            MediaSort::TrendingDesc => "TRENDING_DESC",
            MediaSort::IdDesc => "ID_DESC",
        }
    }
}

// ─── Error Handling ───────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum BotError {
    #[error("API Error: {0}")]
    Api(#[from] crate::api::anilist::AniListError),
    #[error("Network Error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Serenity Error: {0}")]
    Discord(#[from] poise::serenity_prelude::Error),
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON Error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Scheduler Error: {0}")]
    Scheduler(#[from] tokio_cron_scheduler::JobSchedulerError),
    #[error("Internal Error: {0}")]
    Internal(String),
}

impl From<&str> for BotError {
    fn from(s: &str) -> Self {
        BotError::Internal(s.to_string())
    }
}

impl From<String> for BotError {
    fn from(s: String) -> Self {
        BotError::Internal(s)
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for BotError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        BotError::Internal(err.to_string())
    }
}

// ─── Poise type aliases ───────────────────────────────────────────────────────

pub type Error = BotError;
pub type Context<'a> = poise::Context<'a, Data, Error>;
