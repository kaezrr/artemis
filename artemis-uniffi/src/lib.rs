uniffi::setup_scaffolding!();

use std::time::Duration;

use artemis::media::Collection;
use artemis::media::LibraryEntry;
use artemis::media::LibraryItem;
use artemis::media::Media;
use artemis::media::MediaKind;
use artemis::media::ProviderMetadata;
use artemis::media::SearchResult;
use artemis::media::Status;
use artemis::media::UtcDateTime;

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
