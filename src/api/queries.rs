// ─── AniList GraphQL Query Strings ───────────────────────────────────────────
//
// All schema knowledge lives here. Changing a field name or adding a new one
// only requires editing this file and the corresponding response struct.

/// Search anime — returns up to 5 results for pagination.
pub const ANIME_SEARCH_QUERY: &str = r#"
query ($search: String) {
  Page(perPage: 5) {
    pageInfo { total currentPage lastPage hasNextPage }
    media(search: $search, type: ANIME) {
      id
      title { romaji english native }
      episodes season seasonYear format status averageScore genres
      description(asHtml: false)
      coverImage { large }
      siteUrl
      startDate { year month day }
    }
  }
}
"#;

/// Search manga — returns up to 5 results for pagination.
pub const MANGA_SEARCH_QUERY: &str = r#"
query ($search: String) {
  Page(perPage: 5) {
    pageInfo { total currentPage lastPage hasNextPage }
    media(search: $search, type: MANGA) {
      id
      title { romaji english native }
      chapters volumes format status averageScore genres
      description(asHtml: false)
      coverImage { large }
      siteUrl
      startDate { year month day }
    }
  }
}
"#;

/// Search a character by name.
pub const CHARACTER_QUERY: &str = r#"
query ($search: String) {
  Character(search: $search) {
    id
    name { full native }
    description(asHtml: false)
    image { large }
    siteUrl
    media(perPage: 6, sort: POPULARITY_DESC) {
      nodes {
        title { romaji english }
        type
        siteUrl
        coverImage { large }
      }
    }
  }
}
"#;

/// Search a studio by name.
pub const STUDIO_QUERY: &str = r#"
query ($search: String) {
  Studio(search: $search) {
    id
    name
    siteUrl
    isAnimationStudio
    media(sort: POPULARITY_DESC, isMain: true, perPage: 8) {
      nodes {
        title { romaji english }
        seasonYear averageScore format
        siteUrl
      }
    }
  }
}
"#;

/// Upcoming anime for a given season and year.
pub const UPCOMING_QUERY: &str = r#"
query ($season: MediaSeason, $seasonYear: Int) {
  Page(perPage: 10) {
    pageInfo { total currentPage lastPage hasNextPage }
    media(season: $season, seasonYear: $seasonYear, type: ANIME, sort: POPULARITY_DESC) {
      id
      title { romaji english native }
      episodes format genres averageScore
      coverImage { large }
      siteUrl
      startDate { year month day }
    }
  }
}
"#;

/// Currently airing anime with next-episode countdown.
pub const AIRING_QUERY: &str = r#"
query {
  Page(perPage: 10) {
    pageInfo { total currentPage lastPage hasNextPage }
    media(type: ANIME, status: RELEASING, sort: POPULARITY_DESC) {
      id
      title { romaji english native }
      episodes format averageScore
      coverImage { large }
      siteUrl
      nextAiringEpisode { airingAt episode timeUntilAiring }
    }
  }
}
"#;

/// Fetch one entry from a specific page for random selection.
pub const RANDOM_PAGE_QUERY: &str = r#"
query ($type: MediaType, $page: Int) {
  Page(page: $page, perPage: 1) {
    pageInfo { total currentPage lastPage hasNextPage }
    media(type: $type, sort: POPULARITY_DESC, averageScore_greater: 65, popularity_greater: 1000) {
      id
      title { romaji english native }
      episodes chapters volumes season seasonYear format status averageScore genres
      description(asHtml: false)
      coverImage { large }
      siteUrl
      startDate { year month day }
    }
  }
}
"#;

/// AniList user profile.
pub const USER_QUERY: &str = r#"
query ($name: String) {
  User(name: $name) {
    id name
    about(asHtml: false)
    avatar { large }
    siteUrl
    statistics {
      anime { count episodesWatched minutesWatched meanScore }
      manga { count chaptersRead meanScore }
    }
  }
}
"#;