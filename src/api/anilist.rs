use reqwest::Client;
use serde_json::json;

use crate::models::responses::{GraphQlResponse, Media, MediaData, AniListUser, UserData};
use super::queries;

const ANILIST_URL: &str = "https://graphql.anilist.co";

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Fire a POST to the AniList GraphQL endpoint with the given query + variables,
/// then deserialise the response into `T`.
///
/// This is the single place that touches the network — every public function
/// below delegates here.
async fn graphql_post<T>(
    client: &Client,
    query: &str,
    variables: serde_json::Value,
) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
where
    T: serde::de::DeserializeOwned,
{
    let body = json!({ "query": query, "variables": variables });

    let response = client
        .post(ANILIST_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&body)
        .send()
        .await?;

    // Surface HTTP-level errors before attempting to deserialise.
    let response = response.error_for_status()?;

    let parsed: GraphQlResponse<T> = response.json().await?;
    Ok(parsed.data)
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Search AniList for an anime by title.
/// Returns the best-matching `Media` struct or an error.
pub async fn fetch_anime(
    client: &Client,
    search: &str,
) -> Result<Media, Box<dyn std::error::Error + Send + Sync>> {
    let vars = json!({ "search": search });
    let data: MediaData = graphql_post(client, queries::ANIME_QUERY, vars).await?;
    Ok(data.media)
}

/// Search AniList for a manga by title.
pub async fn fetch_manga(
    client: &Client,
    search: &str,
) -> Result<Media, Box<dyn std::error::Error + Send + Sync>> {
    let vars = json!({ "search": search });
    let data: MediaData = graphql_post(client, queries::MANGA_QUERY, vars).await?;
    Ok(data.media)
}

/// Fetch a public AniList user profile by username.
pub async fn fetch_user(
    client: &Client,
    username: &str,
) -> Result<AniListUser, Box<dyn std::error::Error + Send + Sync>> {
    let vars = json!({ "name": username });
    let data: UserData = graphql_post(client, queries::USER_QUERY, vars).await?;
    Ok(data.user)
}
