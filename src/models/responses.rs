use serde::{Deserialize, Serialize};

// ─── GraphQL envelope ────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphQlResponse<T> {
    pub data: T,
}

// ─── Shared primitives ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaTitle {
    pub romaji: Option<String>,
    pub english: Option<String>,
    pub native: Option<String>,
}

impl MediaTitle {
    pub fn preferred(&self) -> &str {
        self.english
            .as_deref()
            .or(self.romaji.as_deref())
            .or(self.native.as_deref())
            .unwrap_or("Unknown Title")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverImage {
    pub large: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzyDate {
    pub year: Option<u32>,
    pub month: Option<u32>,
    pub day: Option<u32>,
}

impl FuzzyDate {
    pub fn display(&self) -> String {
        match (self.year, self.month, self.day) {
            (Some(y), Some(m), Some(d)) => format!("{y}-{m:02}-{d:02}"),
            (Some(y), Some(m), None)    => format!("{y}-{m:02}"),
            (Some(y), None,    None)    => format!("{y}"),
            _                           => "Unknown".to_string(),
        }
    }
}

// ─── Media (Anime / Manga / Upcoming / Airing / Random) ──────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Media {
    pub id: u64,
    pub title: MediaTitle,

    // Anime
    pub episodes: Option<u32>,
    pub season: Option<String>,
    #[serde(rename = "seasonYear")]
    pub season_year: Option<u32>,

    // Manga
    pub chapters: Option<u32>,
    pub volumes: Option<u32>,

    // Shared
    pub format: Option<String>,
    pub status: Option<String>,
    #[serde(rename = "averageScore")]
    pub average_score: Option<u32>,
    #[serde(default)]
    pub genres: Vec<String>,
    pub description: Option<String>,
    #[serde(rename = "coverImage")]
    pub cover_image: CoverImage,
    #[serde(rename = "siteUrl")]
    pub site_url: String,
    #[serde(rename = "startDate", default)]
    pub start_date: Option<FuzzyDate>,

    // Airing-only (None for non-airing queries)
    #[serde(rename = "nextAiringEpisode")]
    pub next_airing_episode: Option<NextAiringEpisode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NextAiringEpisode {
    #[serde(rename = "airingAt")]
    pub airing_at: i64,
    pub episode: u32,
    #[serde(rename = "timeUntilAiring")]
    pub time_until_airing: i64,
}

impl NextAiringEpisode {
    /// Format seconds-until-airing as "Xd Yh Zm"
    pub fn countdown(&self) -> String {
        let total = self.time_until_airing.max(0) as u64;
        let days    = total / 86400;
        let hours   = (total % 86400) / 3600;
        let minutes = (total % 3600) / 60;
        match (days, hours) {
            (0, 0) => format!("{minutes}m"),
            (0, h)  => format!("{h}h {minutes}m"),
            (d, h)  => format!("{d}d {h}h"),
        }
    }
}

// ─── Paginated media response ─────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct MediaSearchData {
    #[serde(rename = "Page")]
    pub page: MediaPage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MediaPage {
    #[serde(rename = "pageInfo")]
    pub page_info: PageInfo,
    pub media: Vec<Media>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageInfo {
    pub total: Option<u32>,
    #[serde(rename = "currentPage")]
    pub current_page: Option<u32>,
    #[serde(rename = "lastPage")]
    pub last_page: Option<u32>,
    #[serde(rename = "hasNextPage")]
    pub has_next_page: Option<bool>,
}

// ─── Character ───────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct CharacterData {
    #[serde(rename = "Character")]
    pub character: Character,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Character {
    pub id: u64,
    pub name: CharacterName,
    pub description: Option<String>,
    pub image: CharacterImage,
    #[serde(rename = "siteUrl")]
    pub site_url: String,
    pub media: CharacterMediaConnection,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CharacterName {
    pub full: Option<String>,
    pub native: Option<String>,
}

impl CharacterName {
    pub fn preferred(&self) -> &str {
        self.full.as_deref().unwrap_or("Unknown")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CharacterImage {
    pub large: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CharacterMediaConnection {
    pub nodes: Vec<CharacterMediaNode>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CharacterMediaNode {
    pub title: MediaTitle,
    #[serde(rename = "type")]
    pub media_type: Option<String>,
    #[serde(rename = "siteUrl")]
    pub site_url: String,
    #[serde(rename = "coverImage")]
    pub cover_image: CoverImage,
}

// ─── Studio ──────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct StudioData {
    #[serde(rename = "Studio")]
    pub studio: Studio,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Studio {
    pub id: u64,
    pub name: String,
    #[serde(rename = "siteUrl")]
    pub site_url: String,
    #[serde(rename = "isAnimationStudio")]
    pub is_animation_studio: bool,
    pub media: StudioMediaConnection,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StudioMediaConnection {
    pub nodes: Vec<StudioMediaNode>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StudioMediaNode {
    pub title: MediaTitle,
    #[serde(rename = "seasonYear")]
    pub season_year: Option<u32>,
    #[serde(rename = "averageScore")]
    pub average_score: Option<u32>,
    pub format: Option<String>,
    #[serde(rename = "siteUrl")]
    pub site_url: String,
}

// ─── User Profile ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct UserData {
    #[serde(rename = "User")]
    pub user: AniListUser,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AniListUser {
    pub id: u64,
    pub name: String,
    pub about: Option<String>,
    pub avatar: UserAvatar,
    #[serde(rename = "siteUrl")]
    pub site_url: String,
    pub statistics: UserStatistics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserAvatar {
    pub large: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserStatistics {
    pub anime: AnimeStats,
    pub manga: MangaStats,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnimeStats {
    pub count: u32,
    #[serde(rename = "episodesWatched")]
    pub episodes_watched: u32,
    #[serde(rename = "minutesWatched")]
    pub minutes_watched: u32,
    #[serde(rename = "meanScore")]
    pub mean_score: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MangaStats {
    pub count: u32,
    #[serde(rename = "chaptersRead")]
    pub chapters_read: u32,
    #[serde(rename = "meanScore")]
    pub mean_score: f32,
}

// ─── AniList error envelope ───────────────────────────────────────────────────
//
// AniList returns errors as `{ "data": null, "errors": [{ "message", "status" }] }`
// alongside a 200 or 4xx HTTP status.  We parse this so we can surface the
// actual API message instead of a generic decode error.

#[derive(Debug, Deserialize)]
pub struct AniListErrorResponse {
    pub errors: Vec<AniListApiError>,
}

#[derive(Debug, Deserialize)]
pub struct AniListApiError {
    pub message: String,
    pub status: Option<u16>,
}