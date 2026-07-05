mod newtype;

pub use newtype::Duration;
pub use newtype::UtcDateTime;

#[derive(Debug)]
pub struct LibraryEntry {
    pub id: i64,

    pub media: Media,
    pub metadata: ProviderMetadata,

    pub rating: Option<u8>,
    pub notes: Option<String>,
    pub status: Status,

    pub created_at: UtcDateTime,
    pub updated_at: UtcDateTime,
}

/// A light-weight representation of an library entry,
/// contains everything needed to display it in a grid
#[derive(Debug, sqlx::FromRow)]
pub struct LibraryItem {
    pub id: i64,

    pub kind: MediaKind,
    pub title: String,
    pub cover_url: String,

    pub status: Status,
    pub rating: Option<u8>,
}

#[derive(Debug)]
pub struct ProviderMetadata {
    pub provider: String,
    pub provider_id: i64,

    pub title: String,
    pub cover_url: Option<String>,
    pub wide_url: Option<String>,

    pub description: Option<String>,
    pub tags: Vec<String>,
    pub release_year: Option<u32>,
}

#[derive(Debug)]
pub struct SearchResult {
    pub media: Media,
    pub metadata: ProviderMetadata,
    pub in_library: bool,
}

#[derive(sqlx::Type, Clone, Copy)]
#[derive(Default, Debug)]
#[sqlx(rename_all = "snake_case")]
pub enum Status {
    #[default]
    Planned,
    InProgress,
    Finished,
    OnHold,
    Dropped,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Collection {
    pub id: i64,
    pub title: String,
    pub count: i64,
}

#[derive(strum::EnumDiscriminants, Debug)]
#[strum_discriminants(name(MediaKind))]
#[strum_discriminants(derive(sqlx::Type, strum::EnumString))]
#[strum_discriminants(sqlx(rename_all = "snake_case"))]
#[strum_discriminants(doc = "This is a discriminant type without the associated data")]
pub enum Media {
    Anime {
        studio: Option<String>,
        episodes: Option<u32>,
    },

    Movie {
        director: Option<String>,
        duration: Option<Duration>,
    },

    Game {
        developer: Option<String>,
        playtime: Option<Duration>,
    },

    TVShow {
        creator: Option<String>,
        episodes: Option<u32>,
    },
}
