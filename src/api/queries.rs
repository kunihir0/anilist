// ─── AniList GraphQL Query Strings ───────────────────────────────────────────
//
// These are the raw query strings sent to https://graphql.anilist.co.
// Keeping them here isolates all schema knowledge in one place — if AniList
// adds or renames a field, only this file needs to change.

/// Fetch a single anime by its search title.
pub const ANIME_QUERY: &str = r#"
query ($search: String) {
  Media(search: $search, type: ANIME) {
    id
    title {
      romaji
      english
      native
    }
    episodes
    season
    seasonYear
    format
    status
    averageScore
    genres
    description(asHtml: false)
    coverImage { large }
    siteUrl
    startDate { year month day }
  }
}
"#;

/// Fetch a single manga by its search title.
pub const MANGA_QUERY: &str = r#"
query ($search: String) {
  Media(search: $search, type: MANGA) {
    id
    title {
      romaji
      english
      native
    }
    chapters
    volumes
    format
    status
    averageScore
    genres
    description(asHtml: false)
    coverImage { large }
    siteUrl
    startDate { year month day }
  }
}
"#;

/// Fetch an AniList user profile by username.
pub const USER_QUERY: &str = r#"
query ($name: String) {
  User(name: $name) {
    id
    name
    about(asHtml: false)
    avatar { large }
    siteUrl
    statistics {
      anime {
        count
        episodesWatched
        minutesWatched
        meanScore
      }
      manga {
        count
        chaptersRead
        meanScore
      }
    }
  }
}
"#;
