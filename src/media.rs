use time::Duration;
use time::UtcDateTime;

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

pub struct ProviderMetadata {
    pub provider: String,
    pub provider_id: i64,

    pub title: String,
    pub cover_url: String,
    pub wide_url: Option<String>,
    pub logo_url: Option<String>,

    pub description: String,
    pub tags: Vec<Tag>,
    pub release_year: Option<u32>,
}

pub struct SearchResult {
    pub media: Media,
    pub metadata: ProviderMetadata,
}

#[derive(Default)]
pub enum Status {
    #[default]
    Planned,
    InProgress,
    Finished,
    OnHold,
    Dropped,
}

pub struct Collection {
    pub id: i64,
    pub title: String,
    pub entries: Vec<i64>,
}

#[derive(strum::EnumDiscriminants)]
#[strum_discriminants(name(MediaKind))]
#[strum_discriminants(doc = "This is a discriminant type without the associated data")]
pub enum Media {
    Anime {
        episodes: u32,
        studio: String,
    },

    Movie {
        director: String,
        duration: Duration,
    },

    Game {
        developer: String,
        playtime: Option<Duration>,
    },

    TVShow {
        director: String,
        episodes: u32,
    },
}

#[repr(transparent)]
pub struct Tag(String);
