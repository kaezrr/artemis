use time::Duration;
use time::UtcDateTime;

pub struct LibraryEntry {
    pub id: i64,

    pub metadata: ProviderMetadata,
    pub media: Media,

    pub status: Status,
    pub notes: Option<String>,
    pub rating: Option<u8>,

    pub created_at: UtcDateTime,
    pub updated_at: UtcDateTime,
}

pub struct ProviderMetadata {
    pub provider: Provider,
    pub provider_id: i64,

    pub title: String,
    pub cover_url: String,

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
    // Every collection has a unique title
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

    Game {
        playtime: Option<Duration>,
        studio: String,
    },

    Movie {
        director: String,
        duration: Duration,
    },

    TVShow {
        director: String,
        episodes: u32,
    },
}

#[derive(Clone, Copy)]
pub enum Provider {
    Anilist,
    Igdb,
    Tmdb,
}

#[repr(transparent)]
pub struct Tag(String);

impl Tag {
    #[expect(clippy::match_single_binding)]
    pub fn from(provider: Provider, tag: &str) -> Self {
        match tag.trim() {
            other => Self(other.into()),
        }
    }
}
