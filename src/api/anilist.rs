use reqwest::Client;
use serde_json::json;

use super::cache::{Cache, RateLimiter};
use super::queries;
use crate::models::responses::{
    AniListApiError, AniListErrorResponse, AniListUser, Character, CharacterData, FavouritesData,
    GenreCollectionData, GraphQlResponse, Media, MediaListCollection, MediaListCollectionData,
    MediaRecommendationInfo, MediaSearchData, RecommendationData, Staff, StaffBirthday,
    StaffBirthdayData, StaffData, Studio, StudioData, UserData, UserFavourites,
};

const ANILIST_URL: &str = "https://graphql.anilist.co";

type Error = crate::models::bot_data::BotError;

// ─── Core HTTP helper ─────────────────────────────────────────────────────────

/// A typed error that carries both a user-facing message and an optional
/// HTTP/GraphQL status code for richer error embeds.
#[derive(Debug, thiserror::Error)]
pub enum AniListError {
    #[error("API Error: {message}")]
    Api {
        message: String,
        status: Option<u16>,
    },
    #[error("Not Found: {message}")]
    NotFound { message: String },
    #[error("Rate Limit Exceeded")]
    RateLimit,
    #[error("Network Error: {0}")]
    Network(String),
    #[error("Decode Error: {0}")]
    Decode(String),
}

impl From<AniListApiError> for AniListError {
    fn from(e: AniListApiError) -> Self {
        match e.status {
            Some(404) => AniListError::NotFound { message: e.message },
            Some(429) => AniListError::RateLimit,
            _ => AniListError::Api {
                message: e.message,
                status: e.status,
            },
        }
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
        .map_err(|e| AniListError::Network(e.to_string()))?;

    let status_code = resp.status().as_u16();
    let resp = resp.error_for_status().map_err(|e| {
        if status_code == 429 {
            AniListError::RateLimit
        } else {
            AniListError::Api {
                message: e.to_string(),
                status: Some(status_code),
            }
        }
    })?;

    let bytes = resp
        .bytes()
        .await
        .map_err(|e| AniListError::Network(e.to_string()))?;

    if let Ok(err_resp) = serde_json::from_slice::<AniListErrorResponse>(&bytes)
        && let Some(first) = err_resp.errors.into_iter().next()
    {
        return Err(AniListError::from(first).into());
    }

    let parsed: GraphQlResponse<T> =
        serde_json::from_slice(&bytes).map_err(|e| AniListError::Decode(e.to_string()))?;

    Ok(parsed.data)
}

// ─── Generic Caching Helper ───────────────────────────────────────────────────

/// Executes a GraphQL query, checking the cache first.
#[allow(clippy::too_many_arguments)]
async fn fetch_cached<T, R, F>(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
    cache_key: String,
    query: &str,
    variables: serde_json::Value,
    extract_data: F,
    ttl: Option<u64>,
) -> Result<R, Error>
where
    T: serde::de::DeserializeOwned,
    R: serde::Serialize + serde::de::DeserializeOwned + Clone,
    F: FnOnce(T) -> R,
{
    if let Some(cached) = cache.get::<R>(&cache_key).await {
        return Ok(cached);
    }

    let raw_data: T = graphql_post(client, rate_limiter, query, variables).await?;
    let extracted = extract_data(raw_data);

    if let Some(ttl_val) = ttl {
        cache.set_with_ttl(cache_key, &extracted, ttl_val).await;
    } else {
        cache.set(cache_key, &extracted).await;
    }

    Ok(extracted)
}

// ─── Public API ───────────────────────────────────────────────────────────────

pub async fn fetch_anime(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
    search: &str,
) -> Result<Vec<Media>, Error> {
    fetch_cached(
        client,
        cache,
        rate_limiter,
        format!("anime:{search}"),
        queries::ANIME_SEARCH_QUERY,
        json!({ "search": search }),
        |d: MediaSearchData| d.page.media,
        None,
    )
    .await
}

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
    let data: MediaSearchData = graphql_post(
        client,
        rate_limiter,
        queries::ANIME_SEARCH_QUERY,
        json!({ "search": search }),
    )
    .await?;
    let media = data
        .page
        .media
        .into_iter()
        .next()
        .ok_or("Media not found")?;
    cache.set(key, &media).await;
    Ok(media)
}

pub async fn fetch_manga(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
    search: &str,
) -> Result<Vec<Media>, Error> {
    fetch_cached(
        client,
        cache,
        rate_limiter,
        format!("manga:{search}"),
        queries::MANGA_SEARCH_QUERY,
        json!({ "search": search }),
        |d: MediaSearchData| d.page.media,
        None,
    )
    .await
}

pub async fn fetch_character(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
    search: &str,
) -> Result<Character, Error> {
    fetch_cached(
        client,
        cache,
        rate_limiter,
        format!("character:{search}"),
        queries::CHARACTER_QUERY,
        json!({ "search": search }),
        |d: CharacterData| d.character,
        None,
    )
    .await
}

pub async fn fetch_studio(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
    search: &str,
) -> Result<Studio, Error> {
    fetch_cached(
        client,
        cache,
        rate_limiter,
        format!("studio:{search}"),
        queries::STUDIO_QUERY,
        json!({ "search": search }),
        |d: StudioData| d.studio,
        None,
    )
    .await
}

pub async fn fetch_media_characters_by_id(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
    media_id: u64,
) -> Result<crate::models::responses::Media, Error> {
    fetch_cached(
        client,
        cache,
        rate_limiter,
        format!("media:characters:{}", media_id),
        queries::MEDIA_CHARACTERS_BY_ID_QUERY,
        json!({ "id": media_id }),
        |d: crate::models::responses::MediaData| d.media,
        None,
    )
    .await
}

pub async fn fetch_media_recommendations_by_id(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
    media_id: u64,
) -> Result<crate::models::responses::Media, Error> {
    fetch_cached(
        client,
        cache,
        rate_limiter,
        format!("media:recommendations:{}", media_id),
        queries::MEDIA_RECOMMENDATIONS_BY_ID_QUERY,
        json!({ "id": media_id }),
        |d: crate::models::responses::MediaData| d.media,
        None,
    )
    .await
}

pub async fn fetch_upcoming(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
    season: &str,
    year: i32,
) -> Result<Vec<Media>, Error> {
    fetch_cached(
        client,
        cache,
        rate_limiter,
        format!("upcoming:{season}:{year}"),
        queries::UPCOMING_QUERY,
        json!({ "season": season, "seasonYear": year }),
        |d: MediaSearchData| d.page.media,
        None,
    )
    .await
}

pub async fn fetch_airing(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
) -> Result<Vec<Media>, Error> {
    fetch_cached(
        client,
        cache,
        rate_limiter,
        "airing:current".to_string(),
        queries::AIRING_QUERY,
        json!({ "type": "ANIME" }),
        |d: MediaSearchData| d.page.media,
        None,
    )
    .await
}

pub async fn fetch_random(
    client: &Client,
    rate_limiter: &RateLimiter,
    media_type: &str,
) -> Result<Media, Error> {
    let page = rand_page(100);
    let vars = json!({ "type": media_type, "page": page });
    let data: MediaSearchData =
        graphql_post(client, rate_limiter, queries::RANDOM_PAGE_QUERY, vars).await?;

    if let Some(media) = data.page.media.into_iter().next() {
        return Ok(media);
    }

    let fallback_data: MediaSearchData = graphql_post(
        client,
        rate_limiter,
        queries::RANDOM_PAGE_QUERY,
        json!({ "type": media_type, "page": 1 }),
    )
    .await?;
    fallback_data
        .page
        .media
        .into_iter()
        .next()
        .ok_or_else(|| "No results returned for random query.".into())
}

fn rand_page(max: u32) -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let mut x: u64 = (dur.as_secs().wrapping_mul(1_000_000_007)) ^ (dur.subsec_nanos() as u64);
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    ((x % max as u64) as u32).max(1)
}

pub async fn fetch_user(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
    username: &str,
) -> Result<AniListUser, Error> {
    fetch_cached(
        client,
        cache,
        rate_limiter,
        format!("user:{username}"),
        queries::USER_QUERY,
        json!({ "name": username }),
        |d: UserData| d.user,
        None,
    )
    .await
}

pub async fn fetch_staff(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
    search: &str,
) -> Result<Staff, Error> {
    fetch_cached(
        client,
        cache,
        rate_limiter,
        format!("staff:{search}"),
        queries::STAFF_QUERY,
        json!({ "search": search }),
        |d: StaffData| d.staff,
        None,
    )
    .await
}

pub async fn fetch_recommendations(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
    search: &str,
) -> Result<MediaRecommendationInfo, Error> {
    fetch_cached(
        client,
        cache,
        rate_limiter,
        format!("recommendations:{search}"),
        queries::RECOMMENDATIONS_QUERY,
        json!({ "search": search }),
        |d: RecommendationData| d.media,
        None,
    )
    .await
}

pub async fn fetch_trending(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
    media_type: &str,
) -> Result<Vec<Media>, Error> {
    fetch_cached(
        client,
        cache,
        rate_limiter,
        format!("trending:{media_type}"),
        queries::TRENDING_QUERY,
        json!({ "type": media_type }),
        |d: MediaSearchData| d.page.media,
        None,
    )
    .await
}

pub async fn fetch_genre(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
    genre: &str,
    media_type: &str,
) -> Result<Vec<Media>, Error> {
    fetch_cached(
        client,
        cache,
        rate_limiter,
        format!("genre:{genre}:{media_type}"),
        queries::GENRE_QUERY,
        json!({ "genre": genre, "type": media_type }),
        |d: MediaSearchData| d.page.media,
        None,
    )
    .await
}

pub async fn fetch_tags(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
) -> Result<Vec<String>, Error> {
    fetch_cached(
        client,
        cache,
        rate_limiter,
        "tags:all".to_string(),
        queries::TAG_COLLECTION_QUERY,
        json!({}),
        |d: crate::models::responses::TagCollectionData| d.tags.into_iter().map(|t| t.name).collect(),
        Some(86400 * 7), // 1 week
    )
    .await
}

pub async fn fetch_by_tag(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
    tag: &str,
    media_type: &str,
) -> Result<Vec<Media>, Error> {
    fetch_cached(
        client,
        cache,
        rate_limiter,
        format!("tag:{tag}:{media_type}"),
        queries::TAG_QUERY,
        json!({ "tag": tag, "type": media_type }),
        |d: MediaSearchData| d.page.media,
        None,
    )
    .await
}

pub async fn fetch_favourites(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
    username: &str,
) -> Result<UserFavourites, Error> {
    fetch_cached(
        client,
        cache,
        rate_limiter,
        format!("favourites:{username}"),
        queries::FAVOURITES_QUERY,
        json!({ "name": username }),
        |d: FavouritesData| d.user,
        None,
    )
    .await
}

pub async fn fetch_staff_birthdays(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
) -> Result<Vec<StaffBirthday>, Error> {
    fetch_cached(
        client,
        cache,
        rate_limiter,
        "staff:birthdays:today".to_string(),
        queries::STAFF_BIRTHDAY_QUERY,
        json!({}),
        |d: StaffBirthdayData| d.page.staff,
        Some(3600),
    )
    .await
}

pub async fn fetch_watchlist(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
    username: &str,
    media_type: &str,
) -> Result<MediaListCollection, Error> {
    fetch_cached(
        client,
        cache,
        rate_limiter,
        format!("watchlist:{username}:{media_type}"),
        queries::MEDIA_LIST_QUERY,
        json!({ "name": username, "type": media_type }),
        |d: MediaListCollectionData| d.collection,
        None,
    )
    .await
}

pub async fn fetch_genres(
    client: &Client,
    cache: &Cache,
    rate_limiter: &RateLimiter,
) -> Result<Vec<String>, Error> {
    fetch_cached(
        client,
        cache,
        rate_limiter,
        "genres:collection".to_string(),
        queries::GENRE_COLLECTION_QUERY,
        json!({}),
        |d: GenreCollectionData| d.genres,
        Some(86400),
    )
    .await
}

#[allow(clippy::too_many_arguments)]
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
    let data: MediaSearchData =
        graphql_post(client, rate_limiter, queries::FILTER_QUERY, vars).await?;
    Ok(data.page.media)
}

// ─── Autocomplete helpers ─────────────────────────────────────────────────────

use crate::models::responses::{
    AutocompleteMediaData, AutocompleteMediaItem, AutocompleteNameData, AutocompleteNameItem,
    AutocompleteStudioData, AutocompleteStudioItem,
};

pub async fn fetch_media_autocomplete(
    client: &Client,
    rate_limiter: &RateLimiter,
    search: &str,
    media_type: &str,
) -> Result<Vec<AutocompleteMediaItem>, Error> {
    let data: AutocompleteMediaData = graphql_post(
        client,
        rate_limiter,
        queries::MEDIA_AUTOCOMPLETE_QUERY,
        json!({ "search": search, "type": media_type }),
    )
    .await?;
    Ok(data.page.media)
}

pub async fn fetch_character_autocomplete(
    client: &Client,
    rate_limiter: &RateLimiter,
    search: &str,
) -> Result<Vec<AutocompleteNameItem>, Error> {
    let data: AutocompleteNameData = graphql_post(
        client,
        rate_limiter,
        queries::CHARACTER_AUTOCOMPLETE_QUERY,
        json!({ "search": search }),
    )
    .await?;
    Ok(data.page.characters.unwrap_or_default())
}

pub async fn fetch_staff_autocomplete(
    client: &Client,
    rate_limiter: &RateLimiter,
    search: &str,
) -> Result<Vec<AutocompleteNameItem>, Error> {
    let data: AutocompleteNameData = graphql_post(
        client,
        rate_limiter,
        queries::STAFF_AUTOCOMPLETE_QUERY,
        json!({ "search": search }),
    )
    .await?;
    Ok(data.page.staff.unwrap_or_default())
}

pub async fn fetch_studio_autocomplete(
    client: &Client,
    rate_limiter: &RateLimiter,
    search: &str,
) -> Result<Vec<AutocompleteStudioItem>, Error> {
    let data: AutocompleteStudioData = graphql_post(
        client,
        rate_limiter,
        queries::STUDIO_AUTOCOMPLETE_QUERY,
        json!({ "search": search }),
    )
    .await?;
    Ok(data.page.studios)
}
