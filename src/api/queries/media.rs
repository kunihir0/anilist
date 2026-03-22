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
      relations {
        edges {
          relationType
          node {
            id
            title { romaji english }
            format
            status
            siteUrl
          }
        }
      }
    }
  }
}
"#;

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
      relations {
        edges {
          relationType
          node {
            id
            title { romaji english }
            format
            status
            siteUrl
          }
        }
      }
    }
  }
}
"#;

pub const FILTER_QUERY: &str = r#"
query (
  $page: Int = 1,
  $type: MediaType,
  $format: [MediaFormat],
  $status: MediaStatus,
  $country: CountryCode,
  $genres: [String],
  $year: Int,
  $sort: [MediaSort] = [POPULARITY_DESC]
) {
  Page(page: $page, perPage: 10) {
    pageInfo { total currentPage lastPage hasNextPage }
    media(
      type: $type,
      format_in: $format,
      status: $status,
      countryOfOrigin: $country,
      genre_in: $genres,
      seasonYear: $year,
      sort: $sort
    ) {
      id
      title { romaji english }
      format
      status
      averageScore
      siteUrl
    }
  }
}
"#;

pub const RECOMMENDATIONS_QUERY: &str = r#"
query ($search: String) {
  Media(search: $search) {
    id
    title { romaji english native }
    recommendations(perPage: 5, sort: RATING_DESC) {
      nodes {
        mediaRecommendation {
          title { romaji english }
          siteUrl
        }
      }
    }
  }
}
"#;

pub const TRENDING_QUERY: &str = r#"
query ($type: MediaType) {
  Page(perPage: 10) {
    media(type: $type, sort: [TRENDING_DESC]) {
      id
      title { romaji english }
      siteUrl
      averageScore
    }
  }
}
"#;

pub const GENRE_QUERY: &str = r#"
query ($genre: String, $type: MediaType) {
  Page(perPage: 10) {
    media(genre_in: [$genre], type: $type, sort: [POPULARITY_DESC]) {
      id
      title { romaji english }
      siteUrl
      averageScore
    }
  }
}
"#;

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

pub const AIRING_QUERY: &str = r#"
query ($type: MediaType) {
  Page(perPage: 10) {
    pageInfo { total currentPage lastPage hasNextPage }
    media(type: $type, status: RELEASING, sort: POPULARITY_DESC) {
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

pub const GENRE_COLLECTION_QUERY: &str = r#"
query {
  GenreCollection
}
"#;

pub const TAG_COLLECTION_QUERY: &str = r#"
query {
  MediaTagCollection {
    name
  }
}
"#;

pub const TAG_QUERY: &str = r#"
query ($tag: String, $type: MediaType) {
  Page(perPage: 10) {
    media(tag: $tag, type: $type, sort: [POPULARITY_DESC]) {
      id
      title { romaji english }
      siteUrl
      averageScore
    }
  }
}
"#;

pub const MEDIA_CHARACTERS_BY_ID_QUERY: &str = r#"
query ($id: Int) {
  Media(id: $id) {
    id
    characters(perPage: 15, sort: [ROLE, RELEVANCE, ID]) {
      edges {
        role
        node {
          id
          name { full }
          siteUrl
        }
      }
    }
  }
}
"#;

pub const MEDIA_RECOMMENDATIONS_BY_ID_QUERY: &str = r#"
query ($id: Int) {
  Media(id: $id) {
    id
    recommendations(perPage: 5, sort: RATING_DESC) {
      nodes {
        mediaRecommendation {
          title { romaji english }
          siteUrl
        }
      }
    }
  }
}
"#;
