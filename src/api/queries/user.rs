pub const USER_QUERY: &str = r#"
query ($name: String) {
  User(name: $name) {
    id name
    about(asHtml: false)
    avatar { large }
    siteUrl
    statistics {
      anime {
        count episodesWatched minutesWatched meanScore
        genres(limit: 5, sort: COUNT_DESC) { genre count }
      }
      manga { count chaptersRead meanScore }
    }
  }
}
"#;

pub const FAVOURITES_QUERY: &str = r#"
query ($name: String) {
  User(name: $name) {
    name siteUrl
    favourites {
      anime(perPage: 5) { nodes { title { romaji english } siteUrl } }
      manga(perPage: 5) { nodes { title { romaji english } siteUrl } }
      characters(perPage: 5) { nodes { name { full } siteUrl } }
      studios(perPage: 5) { nodes { name siteUrl } }
    }
  }
}
"#;

pub const MEDIA_LIST_QUERY: &str = r#"
query ($name: String, $type: MediaType) {
  MediaListCollection(userName: $name, type: $type) {
    lists {
      name
      entries {
        status
        score(format: POINT_100)
        progress
        media {
          id
          title { romaji english }
          siteUrl
        }
      }
    }
  }
}
"#;
