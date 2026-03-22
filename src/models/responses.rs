use crate::store::TitleLanguage;
use serde::{Deserialize, Serialize};

// ─── GraphQL envelope ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    pub fn get_title(&self, lang: Option<TitleLanguage>) -> &str {
        match lang {
            Some(TitleLanguage::Romaji) => self.romaji(),
            Some(TitleLanguage::English) => self.english(),
            Some(TitleLanguage::Native) => self.native(),
            None => self.preferred(),
        }
    }

    pub fn romaji(&self) -> &str {
        self.romaji.as_deref().unwrap_or("Unknown Title")
    }

    pub fn english(&self) -> &str {
        self.english.as_deref().unwrap_or("Unknown Title")
    }

    pub fn native(&self) -> &str {
        self.native.as_deref().unwrap_or("Unknown Title")
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
            (Some(y), Some(m), None) => format!("{y}-{m:02}"),
            (Some(y), None, None) => format!("{y}"),
            _ => "Unknown".to_string(),
        }
    }
}

// ─── Media (Anime / Manga / Upcoming / Airing / Random / Trending / Genre) ──

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
    pub cover_image: Option<CoverImage>, // Make optional because some queries don't fetch it
    #[serde(rename = "siteUrl")]
    pub site_url: String,
    #[serde(rename = "startDate", default)]
    pub start_date: Option<FuzzyDate>,

    // Airing-only (None for non-airing queries)
    #[serde(rename = "nextAiringEpisode")]
    pub next_airing_episode: Option<NextAiringEpisode>,

    // Relations (None for queries that don't fetch them)
    #[serde(default)]
    pub relations: Option<MediaRelations>,

    // Characters (None for queries that don't fetch them)
    #[serde(default)]
    pub characters: Option<MediaCharacterConnection>,

    // Recommendations (None for queries that don't fetch them)
    #[serde(default)]
    pub recommendations: Option<MediaRecommendationConnection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaCharacterConnection {
    pub edges: Vec<MediaCharacterEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaCharacterEdge {
    pub role: Option<String>,
    pub node: MediaCharacterNode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaCharacterNode {
    pub id: u64,
    pub name: CharacterName,
    #[serde(rename = "siteUrl")]
    pub site_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaRecommendationConnection {
    pub nodes: Vec<MediaRecommendationNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaRecommendationNode {
    #[serde(rename = "mediaRecommendation")]
    pub media_recommendation: Option<MediaRecommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaRecommendation {
    pub title: MediaTitle,
    #[serde(rename = "siteUrl")]
    pub site_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaRelations {
    pub edges: Vec<MediaRelationEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaRelationEdge {
    #[serde(rename = "relationType")]
    pub relation_type: String,
    pub node: MediaRelationNode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaRelationNode {
    pub id: u64,
    pub title: MediaTitle,
    pub format: Option<String>,
    pub status: Option<String>,
    #[serde(rename = "siteUrl")]
    pub site_url: String,
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
        let days = total / 86400;
        let hours = (total % 86400) / 3600;
        let minutes = (total % 3600) / 60;
        match (days, hours) {
            (0, 0) => format!("{minutes}m"),
            (0, h) => format!("{h}h {minutes}m"),
            (d, h) => format!("{d}d {h}h"),
        }
    }
}

// ─── Direct Media response ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaData {
    #[serde(rename = "Media")]
    pub media: Media,
}

// ─── Paginated media response ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaSearchData {
    #[serde(rename = "Page")]
    pub page: MediaPage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaPage {
    #[serde(rename = "pageInfo", default)]
    pub page_info: Option<PageInfo>,
    pub media: Vec<Media>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterData {
    #[serde(rename = "Character")]
    pub character: Character,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub id: u64,
    pub name: CharacterName,
    pub description: Option<String>,
    pub image: CharacterImage,
    #[serde(rename = "siteUrl")]
    pub site_url: String,
    pub media: CharacterMediaConnection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterName {
    pub full: Option<String>,
    pub native: Option<String>,
}

impl CharacterName {
    pub fn preferred(&self) -> &str {
        self.full.as_deref().unwrap_or("Unknown")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterImage {
    pub large: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterMediaConnection {
    #[serde(default)]
    pub nodes: Vec<CharacterMediaNode>,
    #[serde(default)]
    pub edges: Vec<CharacterMediaEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterMediaEdge {
    pub node: CharacterMediaNode,
    #[serde(rename = "voiceActors", default)]
    pub voice_actors: Vec<StaffShort>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffShort {
    pub name: StaffShortName,
    #[serde(rename = "siteUrl")]
    pub site_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffShortName {
    pub full: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterMediaNode {
    pub title: MediaTitle,
    #[serde(rename = "type")]
    pub media_type: Option<String>,
    #[serde(rename = "siteUrl")]
    pub site_url: String,
    #[serde(rename = "coverImage", default)]
    pub cover_image: Option<CoverImage>,
}

// ─── Studio ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudioData {
    #[serde(rename = "Studio")]
    pub studio: Studio,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Studio {
    pub id: u64,
    pub name: String,
    #[serde(rename = "siteUrl")]
    pub site_url: String,
    #[serde(rename = "isAnimationStudio")]
    pub is_animation_studio: bool,
    pub media: StudioMediaConnection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudioMediaConnection {
    pub nodes: Vec<StudioMediaNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

// ─── Staff ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffData {
    #[serde(rename = "Staff")]
    pub staff: Staff,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Staff {
    pub id: u64,
    pub name: StaffNameFull,
    pub description: Option<String>,
    pub image: StaffImage,
    #[serde(rename = "siteUrl")]
    pub site_url: String,
    #[serde(rename = "isBirthday")]
    pub is_birthday: bool,
    #[serde(rename = "staffMedia")]
    pub staff_media: StaffMediaConnection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffNameFull {
    pub full: Option<String>,
    pub native: Option<String>,
}

impl StaffNameFull {
    pub fn preferred(&self) -> &str {
        self.full.as_deref().unwrap_or("Unknown")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffImage {
    pub large: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffMediaConnection {
    pub nodes: Vec<StaffMediaNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffMediaNode {
    pub title: MediaTitle,
    #[serde(rename = "type")]
    pub media_type: Option<String>,
    #[serde(rename = "siteUrl")]
    pub site_url: String,
}

// ─── Recommendations ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationData {
    #[serde(rename = "Media")]
    pub media: MediaRecommendationInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaRecommendationInfo {
    pub id: u64,
    pub title: MediaTitle,
    pub recommendations: RecommendationConnection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationConnection {
    pub nodes: Vec<RecommendationNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationNode {
    #[serde(rename = "mediaRecommendation")]
    pub media_recommendation: Option<MediaRecommendationNodeInner>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaRecommendationNodeInner {
    pub title: MediaTitle,
    #[serde(rename = "siteUrl")]
    pub site_url: String,
}

// ─── Favourites ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavouritesData {
    #[serde(rename = "User")]
    pub user: UserFavourites,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFavourites {
    pub name: String,
    #[serde(rename = "siteUrl")]
    pub site_url: String,
    pub favourites: Favourites,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Favourites {
    pub anime: FavouriteMediaConnection,
    pub manga: FavouriteMediaConnection,
    pub characters: FavouriteCharacterConnection,
    pub studios: FavouriteStudioConnection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavouriteMediaConnection {
    pub nodes: Vec<FavouriteMediaNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavouriteMediaNode {
    pub title: MediaTitle,
    #[serde(rename = "siteUrl")]
    pub site_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavouriteCharacterConnection {
    pub nodes: Vec<FavouriteCharacterNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavouriteCharacterNode {
    pub name: CharacterName,
    #[serde(rename = "siteUrl")]
    pub site_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavouriteStudioConnection {
    pub nodes: Vec<FavouriteStudioNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavouriteStudioNode {
    pub name: String,
    #[serde(rename = "siteUrl")]
    pub site_url: String,
}

// ─── User Profile ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserData {
    #[serde(rename = "User")]
    pub user: AniListUser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListUser {
    pub id: u64,
    pub name: String,
    pub about: Option<String>,
    pub avatar: UserAvatar,
    #[serde(rename = "siteUrl")]
    pub site_url: String,
    pub statistics: UserStatistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAvatar {
    pub large: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStatistics {
    pub anime: AnimeStats,
    pub manga: MangaStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimeStats {
    pub count: u32,
    #[serde(rename = "episodesWatched")]
    pub episodes_watched: u32,
    #[serde(rename = "minutesWatched")]
    pub minutes_watched: u32,
    #[serde(rename = "meanScore")]
    pub mean_score: f32,
    #[serde(default)]
    pub genres: Vec<UserGenreStatistic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGenreStatistic {
    pub genre: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MangaStats {
    pub count: u32,
    #[serde(rename = "chaptersRead")]
    pub chapters_read: u32,
    #[serde(rename = "meanScore")]
    pub mean_score: f32,
}

// ─── Media List Collection ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaListCollectionData {
    #[serde(rename = "MediaListCollection")]
    pub collection: MediaListCollection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaListCollection {
    pub lists: Vec<MediaList>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaList {
    pub name: String,
    pub entries: Vec<MediaListEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaListEntry {
    pub status: String,
    pub score: f32,
    pub progress: u32,
    pub media: MediaListNode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaListNode {
    pub id: u64,
    pub title: MediaTitle,
    #[serde(rename = "siteUrl")]
    pub site_url: String,
}

// ─── Genre Collection ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenreCollectionData {
    #[serde(rename = "GenreCollection")]
    pub genres: Vec<String>,
}

// ─── Staff Birthday ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffBirthdayData {
    #[serde(rename = "Page")]
    pub page: StaffBirthdayPage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffBirthdayPage {
    pub staff: Vec<StaffBirthday>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffBirthday {
    pub id: u64,
    pub name: StaffNameFull,
    pub image: StaffImage,
    pub site_url: String,
}

// ─── Autocomplete response types ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutocompleteMediaItem {
    pub id: u64,
    pub title: MediaTitle,
    pub format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutocompleteMediaPage {
    pub media: Vec<AutocompleteMediaItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutocompleteMediaData {
    #[serde(rename = "Page")]
    pub page: AutocompleteMediaPage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutocompleteNameItem {
    pub id: u64,
    pub name: AutocompleteName,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutocompleteName {
    pub full: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutocompleteNamePage {
    pub characters: Option<Vec<AutocompleteNameItem>>,
    pub staff: Option<Vec<AutocompleteNameItem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutocompleteNameData {
    #[serde(rename = "Page")]
    pub page: AutocompleteNamePage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutocompleteStudioItem {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutocompleteStudioPage {
    pub studios: Vec<AutocompleteStudioItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutocompleteStudioData {
    #[serde(rename = "Page")]
    pub page: AutocompleteStudioPage,
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
