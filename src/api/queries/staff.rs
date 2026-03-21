pub const CHARACTER_QUERY: &str = r#"
query ($search: String) {
  Character(search: $search) {
    id
    name { full native }
    description(asHtml: false)
    image { large }
    siteUrl
    media(perPage: 6, sort: POPULARITY_DESC) {
      edges {
        node {
          title { romaji english }
          type
          siteUrl
        }
        voiceActors(language: JAPANESE) {
          name { full }
          siteUrl
        }
      }
    }
  }
}
"#;

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

pub const STAFF_QUERY: &str = r#"
query ($search: String) {
  Staff(search: $search) {
    id
    name { full native }
    description(asHtml: false)
    image { large }
    siteUrl
    isBirthday
    staffMedia(perPage: 6, sort: POPULARITY_DESC) {
      nodes {
        title { romaji english }
        type
        siteUrl
      }
    }
  }
}
"#;

pub const STAFF_BIRTHDAY_QUERY: &str = r#"
query {
  Page(perPage: 50) {
    staff(isBirthday: true) {
      id
      name { full native }
      image { large }
      site_url: siteUrl
    }
  }
}
"#;
