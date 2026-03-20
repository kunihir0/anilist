use serde::Deserialize;

// ─── GraphQL envelope ────────────────────────────────────────────────────────

/// Root wrapper around every AniList response: `{ "data": T }`.
#[derive(Debug, Deserialize)]
pub struct GraphQlResponse<T> {
    pub data: T,
}

// ─── Media (Anime + Manga) ───────────────────────────────────────────────────

/// Matches `{ "data": { "Media": { … } } }`.
#[derive(Debug, Deserialize)]
pub struct MediaData {
    #[serde(rename = "Media")]
    pub media: Media,
}

/// Core media record shared by anime and manga queries.
#[derive(Debug, Deserialize)]
pub struct Media {
    pub id: u64,
    pub title: MediaTitle,

    // Anime-only fields
    pub episodes: Option<u32>,
    pub season: Option<String>,
    #[serde(rename = "seasonYear")]
    pub season_year: Option<u32>,

    // Manga-only fields
    pub chapters: Option<u32>,
    pub volumes: Option<u32>,

    // Shared
    pub format: Option<String>,
    pub status: Option<String>,
    #[serde(rename = "averageScore")]
    pub average_score: Option<u32>,
    pub genres: Vec<String>,
    pub description: Option<String>,
    #[serde(rename = "coverImage")]
    pub cover_image: CoverImage,
    #[serde(rename = "siteUrl")]
    pub site_url: String,
    #[serde(rename = "startDate")]
    pub start_date: FuzzyDate,
}

#[derive(Debug, Deserialize)]
pub struct MediaTitle {
    pub romaji: Option<String>,
    pub english: Option<String>,
    pub native: Option<String>,
}

impl MediaTitle {
    /// Returns the best available display title: English → Romaji → Native → "Unknown".
    pub fn preferred(&self) -> &str {
        self.english
            .as_deref()
            .or(self.romaji.as_deref())
            .or(self.native.as_deref())
            .unwrap_or("Unknown Title")
    }
}

#[derive(Debug, Deserialize)]
pub struct CoverImage {
    /// Large image URL (used as the embed thumbnail).
    pub large: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FuzzyDate {
    pub year: Option<u32>,
    pub month: Option<u32>,
    pub day: Option<u32>,
}

impl FuzzyDate {
    /// Formats the date as `YYYY-MM-DD`, falling back gracefully for missing parts.
    pub fn display(&self) -> String {
        match (self.year, self.month, self.day) {
            (Some(y), Some(m), Some(d)) => format!("{y}-{m:02}-{d:02}"),
            (Some(y), Some(m), None) => format!("{y}-{m:02}"),
            (Some(y), None, None) => format!("{y}"),
            _ => "Unknown".to_string(),
        }
    }
}

// ─── User Profile ────────────────────────────────────────────────────────────

/// Matches `{ "data": { "User": { … } } }`.
#[derive(Debug, Deserialize)]
pub struct UserData {
    #[serde(rename = "User")]
    pub user: AniListUser,
}

#[derive(Debug, Deserialize)]
pub struct AniListUser {
    pub id: u64,
    pub name: String,
    pub about: Option<String>,
    pub avatar: UserAvatar,
    #[serde(rename = "siteUrl")]
    pub site_url: String,
    pub statistics: UserStatistics,
}

#[derive(Debug, Deserialize)]
pub struct UserAvatar {
    pub large: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UserStatistics {
    pub anime: AnimeStats,
    pub manga: MangaStats,
}

#[derive(Debug, Deserialize)]
pub struct AnimeStats {
    pub count: u32,
    #[serde(rename = "episodesWatched")]
    pub episodes_watched: u32,
    #[serde(rename = "minutesWatched")]
    pub minutes_watched: u32,
    #[serde(rename = "meanScore")]
    pub mean_score: f32,
}

#[derive(Debug, Deserialize)]
pub struct MangaStats {
    pub count: u32,
    #[serde(rename = "chaptersRead")]
    pub chapters_read: u32,
    #[serde(rename = "meanScore")]
    pub mean_score: f32,
}
