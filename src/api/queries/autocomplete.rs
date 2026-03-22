pub const MEDIA_AUTOCOMPLETE_QUERY: &str = r#"
query ($search: String, $type: MediaType) {
  Page(perPage: 10) {
    media(search: $search, type: $type) {
      id
      title { romaji english }
      format
    }
  }
}
"#;

pub const CHARACTER_AUTOCOMPLETE_QUERY: &str = r#"
query ($search: String) {
  Page(perPage: 10) {
    characters(search: $search) {
      id
      name { full }
    }
  }
}
"#;

pub const STAFF_AUTOCOMPLETE_QUERY: &str = r#"
query ($search: String) {
  Page(perPage: 10) {
    staff(search: $search) {
      id
      name { full }
    }
  }
}
"#;

pub const STUDIO_AUTOCOMPLETE_QUERY: &str = r#"
query ($search: String) {
  Page(perPage: 10) {
    studios(search: $search) {
      id
      name
    }
  }
}
"#;
