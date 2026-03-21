use reqwest::Client;
use serde_json::json;

use crate::models::responses::{
    AniListApiError, AniListErrorResponse, AniListUser, Character, CharacterData,
    FavouritesData, GenreCollectionData, GraphQlResponse, Media, MediaListCollectionData,
    MediaRecommendationInfo, MediaSearchData, RecommendationData, Staff, StaffBirthday,
    StaffBirthdayData, StaffData, Studio, StudioData, UserData, UserFavourites, MediaListCollection,
};
use super::queries;
use super::cache::{Cache, RateLimiter};

const ANILIST_URL: &str = "https://graphql.anilist.co";

type Error = Box<dyn std::error::Error + Send + Sync>;

// ─── Core HTTP helper ─────────────────────────────────────────────────────────

/// A typed error that carries both a user-facing message and an optional
/// HTTP/GraphQL status code for richer error embeds.
#[derive(Debug)]
pub struct AniListError {
    pub message: String,
    pub status: Option<u16>,
    pub kind: AniListErrorKind,
}

#[derive(Debug)]
pub enum AniListErrorKind {
    NotFound,
    RateLimit,
    ApiError,
    Network,
    Decode,
}

impl std::fmt::Display for AniListError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AniListError {}

impl From<AniListApiError> for AniListError {
    fn from(e: AniListApiError) -> Self {
        let kind = match e.status {
            Some(404) => AniListErrorKind::NotFound,
            Some(429) => AniListErrorKind::RateLimit,
            _         => AniListErrorKind::ApiError,
        };
        AniListError { message: e.message, status: e.status, kind }
    }
}

async fn graphql_post<T>(
    client: &Client,
    rate_limiter: &RateLimiter,
    query: &str,
    variables: serde_json::Value,
) -> Result<T, Error>
where
    T: serde::de::DeserializeOwned,
{
    rate_limiter.check().await?;

    let body = json!({ "query": query, "variables": variables });

    let resp = client
        .post(ANILIST_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| AniListError {
            message: format!("Network error: {e}"),
            status: e.status().map(|s| s.as_u16()),
            kind: AniListErrorKind::Network,
        })?;

    // Surface HTTP-level errors (4xx / 5xx) before attempting to deserialise.
    let status_code = resp.status().as_u16();
    let resp = resp.error_for_status().map_err(|e| AniListError {
        message: format!("HTTP {status_code}: {e}"),
        status: Some(status_code),
        kind: if status_code == 429 {
            AniListErrorKind::RateLimit
        } else {
            AniListErrorKind::ApiError
        },
    })?;

    // Read the body once so we can inspect it for both data and errors.
    let bytes = resp.bytes().await.map_err(|e| AniListError {
        message: format!("Failed to read response body: {e}"),
        status: None,
        kind: AniListErrorKind::Network,
    })?;

    // Check for GraphQL-level errors first (`{ "data": null, "errors": [...] }`).
    if let Ok(err_resp) = serde_json::from_slice::<AniListErrorResponse>(&bytes) {
        if let Some(first) = err_resp.errors.into_iter().next() {
            return Err(Box::new(AniListError::from(first)));
        }
    }

    // Deserialise the happy-path response.
    let parsed: GraphQlResponse<T> = serde_json::from_slice(&bytes).map_err(|e| AniListError {
        message: format!("Unexpected response format: {e}"),
        status: None,
        kind: AniListErrorKind::Decode,
    })?;

    Ok(parsed.data)
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Search anime by title — returns up to 5 results.
pub async fn fetch_anime(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
    search: &str,
) -> Result<Vec<Media>, Error> {
    let key = format!("anime:{search}");
    if let Some(cached) = cache.get::<Vec<Media>>(&key).await {
        return Ok(cached);
    }
    let vars = json!({ "search": search });
    let data: MediaSearchData =
        graphql_post(client, rate_limiter, queries::ANIME_SEARCH_QUERY, vars).await?;
    let results = data.page.media;
    cache.set(key, &results).await;
    Ok(results)
}

/// Search a single media by title.
pub async fn fetch_media_by_title(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
    search: &str,
) -> Result<Media, Error> {
    let key = format!("media:title:{search}");
    if let Some(cached) = cache.get::<Media>(&key).await {
        return Ok(cached);
    }
    let vars = json!({ "search": search });
    let data: MediaSearchData =
        graphql_post(client, rate_limiter, queries::ANIME_SEARCH_QUERY, vars).await?;
    let media = data.page.media.into_iter().next().ok_or("Media not found")?;
    cache.set(key, &media).await;
    Ok(media)
}

/// Search manga by title — returns up to 5 results.
pub async fn fetch_manga(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
    search: &str,
) -> Result<Vec<Media>, Error> {
    let key = format!("manga:{search}");
    if let Some(cached) = cache.get::<Vec<Media>>(&key).await {
        return Ok(cached);
    }
    let vars = json!({ "search": search });
    let data: MediaSearchData =
        graphql_post(client, rate_limiter, queries::MANGA_SEARCH_QUERY, vars).await?;
    let results = data.page.media;
    cache.set(key, &results).await;
    Ok(results)
}

/// Fetch a character by name.
pub async fn fetch_character(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
    search: &str,
) -> Result<Character, Error> {
    let key = format!("character:{search}");
    if let Some(cached) = cache.get::<Character>(&key).await {
        return Ok(cached);
    }
    let vars = json!({ "search": search });
    let data: CharacterData =
        graphql_post(client, rate_limiter, queries::CHARACTER_QUERY, vars).await?;
    cache.set(key, &data.character).await;
    Ok(data.character)
}

/// Fetch a studio by name.
pub async fn fetch_studio(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
    search: &str,
) -> Result<Studio, Error> {
    let key = format!("studio:{search}");
    if let Some(cached) = cache.get::<Studio>(&key).await {
        return Ok(cached);
    }
    let vars = json!({ "search": search });
    let data: StudioData =
        graphql_post(client, rate_limiter, queries::STUDIO_QUERY, vars).await?;
    cache.set(key, &data.studio).await;
    Ok(data.studio)
}

/// Fetch upcoming anime for a given season + year.
pub async fn fetch_upcoming(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
    season: &str,
    year: i32,
) -> Result<Vec<Media>, Error> {
    let key = format!("upcoming:{season}:{year}");
    if let Some(cached) = cache.get::<Vec<Media>>(&key).await {
        return Ok(cached);
    }
    let vars = json!({ "season": season, "seasonYear": year });
    let data: MediaSearchData =
        graphql_post(client, rate_limiter, queries::UPCOMING_QUERY, vars).await?;
    let results = data.page.media;
    cache.set(key, &results).await;
    Ok(results)
}

/// Fetch currently airing anime with next-episode countdowns.
pub async fn fetch_airing(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
) -> Result<Vec<Media>, Error> {
    let key = "airing:current".to_string();
    // Short TTL for airing data — we still respect the cache set by the
    // caller (2 min TTL in Data::new for airing vs 5 min for everything else).
    if let Some(cached) = cache.get::<Vec<Media>>(&key).await {
        return Ok(cached);
    }
    let vars = json!({ "type": "ANIME" });
    let data: MediaSearchData =
        graphql_post(client, rate_limiter, queries::AIRING_QUERY, vars).await?;
    let results = data.page.media;
    cache.set(key, &results).await;
    Ok(results)
}

/// Fetch a random anime or manga ("ANIME" | "MANGA").
///
/// AniList does not support `sort: RANDOM` and `lastPage` from pageInfo is
/// documented as inaccurate. Instead we pick a random page from a fixed pool
/// of 100 (perPage: 1, popularity-sorted) and fall back to page 1 if the
/// chosen page returns nothing.
pub async fn fetch_random(
    client: &Client,
    rate_limiter: &RateLimiter,
    media_type: &str,
) -> Result<Media, Error> {
    let page = rand_page(100);
    let vars = json!({ "type": media_type, "page": page });

    let data: MediaSearchData =
        graphql_post(client, rate_limiter, queries::RANDOM_PAGE_QUERY, vars).await?;

    // If the chosen page is beyond the actual pool, fall back to page 1.
    if let Some(media) = data.page.media.into_iter().next() {
        return Ok(media);
    }

    let fallback = json!({ "type": media_type, "page": 1 });
    let fallback_data: MediaSearchData =
        graphql_post(client, rate_limiter, queries::RANDOM_PAGE_QUERY, fallback).await?;

    fallback_data
        .page
        .media
        .into_iter()
        .next()
        .ok_or_else(|| "No results returned for random query.".into())
}

/// Pseudo-random page in [1, max] — no extra crate.
///
/// Mixes full seconds and nanoseconds so repeated calls within the same
/// second produce different values. Uses a Xorshift64 step to spread bits.
fn rand_page(max: u32) -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    // Combine whole seconds and nanoseconds into one 64-bit seed.
    let mut x: u64 = (dur.as_secs().wrapping_mul(1_000_000_007))
        ^ (dur.subsec_nanos() as u64);

    // One round of Xorshift64 to spread bits across the full range.
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;

    ((x % max as u64) as u32).max(1)
}

/// Fetch an AniList user profile.
pub async fn fetch_user(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
    username: &str,
) -> Result<AniListUser, Error> {
    let key = format!("user:{username}");
    if let Some(cached) = cache.get::<AniListUser>(&key).await {
        return Ok(cached);
    }
    let vars = json!({ "name": username });
    let data: UserData =
        graphql_post(client, rate_limiter, queries::USER_QUERY, vars).await?;
    cache.set(key, &data.user).await;
    Ok(data.user)
}

/// Fetch a staff by name.
pub async fn fetch_staff(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
    search: &str,
) -> Result<Staff, Error> {
    let key = format!("staff:{search}");
    if let Some(cached) = cache.get::<Staff>(&key).await {
        return Ok(cached);
    }
    let vars = json!({ "search": search });
    let data: StaffData =
        graphql_post(client, rate_limiter, queries::STAFF_QUERY, vars).await?;
    cache.set(key, &data.staff).await;
    Ok(data.staff)
}

/// Fetch recommendations for a media title.
pub async fn fetch_recommendations(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
    search: &str,
) -> Result<MediaRecommendationInfo, Error> {
    let key = format!("recommendations:{search}");
    if let Some(cached) = cache.get::<MediaRecommendationInfo>(&key).await {
        return Ok(cached);
    }
    let vars = json!({ "search": search });
    let data: RecommendationData =
        graphql_post(client, rate_limiter, queries::RECOMMENDATIONS_QUERY, vars).await?;
    cache.set(key, &data.media).await;
    Ok(data.media)
}

/// Fetch trending media.
pub async fn fetch_trending(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
    media_type: &str,
) -> Result<Vec<Media>, Error> {
    let key = format!("trending:{media_type}");
    if let Some(cached) = cache.get::<Vec<Media>>(&key).await {
        return Ok(cached);
    }
    let vars = json!({ "type": media_type });
    let data: MediaSearchData =
        graphql_post(client, rate_limiter, queries::TRENDING_QUERY, vars).await?;
    let results = data.page.media;
    cache.set(key, &results).await;
    Ok(results)
}

/// Fetch media by genre.
pub async fn fetch_genre(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
    genre: &str,
    media_type: &str,
) -> Result<Vec<Media>, Error> {
    let key = format!("genre:{genre}:{media_type}");
    if let Some(cached) = cache.get::<Vec<Media>>(&key).await {
        return Ok(cached);
    }
    let vars = json!({ "genre": genre, "type": media_type });
    let data: MediaSearchData =
        graphql_post(client, rate_limiter, queries::GENRE_QUERY, vars).await?;
    let results = data.page.media;
    cache.set(key, &results).await;
    Ok(results)
}

/// Fetch a user's favourites.
pub async fn fetch_favourites(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
    username: &str,
) -> Result<UserFavourites, Error> {
    let key = format!("favourites:{username}");
    if let Some(cached) = cache.get::<UserFavourites>(&key).await {
        return Ok(cached);
    }
    let vars = json!({ "name": username });
    let data: UserFavourites =
        graphql_post(client, rate_limiter, queries::FAVOURITES_QUERY, vars).await?;
    cache.set(key, &data).await;
    Ok(data)
}

/// Fetch staff members with birthdays today.
pub async fn fetch_staff_birthdays(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
) -> Result<Vec<StaffBirthday>, Error> {
    let key = "staff:birthdays:today".to_string();
    if let Some(cached) = cache.get::<Vec<StaffBirthday>>(&key).await {
        return Ok(cached);
    }
    let data: StaffBirthdayData =
        graphql_post(client, rate_limiter, queries::STAFF_BIRTHDAY_QUERY, json!({})).await?;
    let results = data.page.staff;
    // Cache for 1 hour
    cache.set_with_ttl(key, &results, 3600).await;
    Ok(results)
}

/// Fetch a user's media list collection.
pub async fn fetch_watchlist(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
    username: &str,
    media_type: &str,
) -> Result<MediaListCollection, Error> {
    let key = format!("watchlist:{username}:{media_type}");
    if let Some(cached) = cache.get::<MediaListCollection>(&key).await {
        return Ok(cached);
    }
    let vars = json!({ "name": username, "type": media_type });
    let data: MediaListCollectionData =
        graphql_post(client, rate_limiter, queries::MEDIA_LIST_QUERY, vars).await?;
    cache.set(key, &data.collection).await;
    Ok(data.collection)
}

/// Fetch all available genres from AniList.
pub async fn fetch_genres(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
) -> Result<Vec<String>, Error> {
    let key = "genres:collection".to_string();
    if let Some(cached) = cache.get::<Vec<String>>(&key).await {
        return Ok(cached);
    }
    let data: GenreCollectionData =
        graphql_post(client, rate_limiter, queries::GENRE_COLLECTION_QUERY, json!({})).await?;
    cache.set_with_ttl(key, &data.genres, 86400).await; // Cache for 24h
    Ok(data.genres)
}

/// Fetch filtered media.
pub async fn fetch_filtered_media(
    client: &Client,
    _cache: &Cache,
    rate_limiter: &RateLimiter,
    media_type: Option<&str>,
    format: Option<Vec<&str>>,
    status: Option<&str>,
    country: Option<&str>,
    genres: Option<Vec<&str>>,
    year: Option<i32>,
    sort: Option<Vec<&str>>,
) -> Result<Vec<Media>, Error> {
    let vars = json!({
        "type": media_type,
        "format": format,
        "status": status,
        "country": country,
        "genres": genres,
        "year": year,
        "sort": sort,
    });
    // We don't cache this due to the high number of parameter combinations.
    let data: MediaSearchData =
        graphql_post(client, rate_limiter, queries::FILTER_QUERY, vars).await?;
    Ok(data.page.media)
}
