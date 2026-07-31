uniffi::setup_scaffolding!();

use std::collections::HashMap;
use std::time::Duration;

use artemis::media::*;
use artemis::query::*;

uniffi::custom_type!(UtcDateTime, i64, {
    remote,
    lower:|d| d.unix_timestamp(),
    try_lift: |d| Ok(UtcDateTime::from_unix_timestamp(d)?)
});

#[uniffi::remote(Record)]
struct LibraryEntry {
    id: i64,

    media: Media,
    metadata: ProviderMetadata,

    rating: Option<u8>,
    notes: Option<String>,
    status: Status,

    created_at: UtcDateTime,
    updated_at: UtcDateTime,
}

#[uniffi::remote(Record)]
struct LibraryItem {
    id: i64,

    kind: MediaKind,
    title: String,
    cover_url: String,

    status: Status,
    rating: Option<u8>,
}

#[uniffi::remote(Record)]
struct ProviderMetadata {
    provider: String,
    provider_id: i64,

    title: String,
    cover_url: Option<String>,
    wide_url: Option<String>,

    description: Option<String>,
    tags: Vec<String>,
    release_year: Option<u32>,
}

#[uniffi::remote(Record)]
struct SearchResult {
    media: Media,
    metadata: ProviderMetadata,
    in_library: bool,
}

#[uniffi::remote(Enum)]
enum Status {
    Planned,
    InProgress,
    Finished,
    OnHold,
    Dropped,
}

#[uniffi::remote(Record)]
struct Collection {
    id: i64,
    title: String,
    count: i64,
}

#[uniffi::remote(Enum)]
enum Media {
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

#[uniffi::remote(Enum)]
enum MediaKind {
    Anime,
    Movie,
    Game,
    TVShow,
}

#[uniffi::remote(Record)]
pub struct LibraryQuery {
    pub search: Option<String>,
    pub kind: Option<MediaKind>,

    pub sort_by: SortBy,
    pub order: SortOrder,

    pub status: Option<Status>,
    pub tag_filter: Option<TagFilter>,

    pub limit: Option<u32>,
    pub offset: Option<u32>,

    pub collection_id: Option<i64>,
}

#[uniffi::remote(Record)]
pub struct SearchQuery {
    pub query: String,
    pub kind: Option<MediaKind>,
}

#[uniffi::remote(Enum)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[uniffi::remote(Enum)]
pub enum SortBy {
    Title,
    Rating,
    ReleaseYear,
    LastModified,
}

#[uniffi::remote(Enum)]
pub enum TagFilter {
    Or(Vec<String>),
    And(Vec<String>),
}

#[uniffi::remote(Record)]
pub struct Dashboard {
    pub recent: Vec<LibraryItem>,
    pub media_counts: HashMap<MediaKind, u32>,
}

#[uniffi::remote(Record)]
pub struct UpdateEntry {
    pub status: Option<Status>,
    pub notes: Option<Option<String>>,
    pub rating: Option<Option<u8>>,
    pub playtime: Option<Option<Duration>>,
}

#[uniffi::remote(Record)]
pub struct UpdateCollection {
    pub title: Option<String>,
    pub update_entries: Vec<CollectionAction>,
}

#[uniffi::remote(Enum)]
pub enum CollectionAction {
    Add(i64),
    Remove(i64),
}
