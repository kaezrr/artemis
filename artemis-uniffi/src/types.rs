use std::collections::HashMap;
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
use artemis::query::CollectionAction;
use artemis::query::Dashboard;
use artemis::query::LibraryQuery;
use artemis::query::SortBy;
use artemis::query::SortOrder;
use artemis::query::TagFilter;
use artemis::query::UpdateCollection;
use artemis::query::UpdateEntry;

uniffi::custom_type!(UtcDateTime, i64, {
    remote,
    lower:|d| d.unix_timestamp(),
    try_lift: |d| Ok(UtcDateTime::from_unix_timestamp(d)?)
});

/// A full library entry: the media, its source metadata, and the user's
/// score, notes, and progress status.
///
/// `rating` is the user's personal score, unset until they assign one.
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

/// A lightweight entry (id, kind, title, cover, status, rating) for
/// grid/overview displays, without the full metadata.
#[uniffi::remote(Record)]
struct LibraryItem {
    id: i64,

    kind: MediaKind,
    title: String,
    cover_url: String,

    status: Status,
    rating: Option<u8>,
}

/// Metadata as reported by the source provider.
///
/// A media item is uniquely identified by `provider` + `provider_id`.
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

/// A provider search hit, flagged with whether it is already saved.
///
/// `in_library` is filled in by `Application::mark_search_results`.
#[uniffi::remote(Record)]
struct SearchResult {
    media: Media,
    metadata: ProviderMetadata,
    in_library: bool,
}

/// The user's progress status for an entry. `Planned` is the default for
/// newly added entries.
#[uniffi::remote(Enum)]
enum Status {
    Planned,
    InProgress,
    Finished,
    OnHold,
    Dropped,
}

/// A named, user-curated group of library entries.
///
/// `count` is the number of entries currently in the collection.
#[uniffi::remote(Record)]
struct Collection {
    id: i64,
    title: String,
    count: i64,
}

/// A media entry with kind-specific detail fields.
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

/// The kind of a [`Media`] entry, without the kind-specific data.
#[uniffi::remote(Enum)]
enum MediaKind {
    Anime,
    Movie,
    Game,
    TVShow,
}

/// Filters, sorting, and pagination for browsing the library.
///
/// Defaults to `sort_by: SortBy::Title`, `order: SortOrder::Ascending`.
/// When `kind` or `status` is set, only entries of that kind/status match;
/// `tag_filter` narrows by tags; `limit`/`offset` paginate results.
#[uniffi::remote(Record)]
pub struct LibraryQuery {
    /// Free-text search across titles and metadata.
    pub search: Option<String>,
    pub kind: Option<MediaKind>,

    pub sort_by: SortBy,
    pub order: SortOrder,

    pub status: Option<Status>,
    pub tag_filter: Option<TagFilter>,

    pub limit: Option<u32>,
    pub offset: Option<u32>,

    /// Restrict results to entries in this collection.
    pub collection_id: Option<i64>,
}

/// Sort direction for queries. `Ascending` is the default.
#[uniffi::remote(Enum)]
pub enum SortOrder {
    Ascending,
    Descending,
}

/// Sort key for queries. `Title` is the default.
#[uniffi::remote(Enum)]
pub enum SortBy {
    Title,
    Rating,
    ReleaseYear,
    LastModified,
}

/// Tag filtering: `Or` matches entries with any of the listed tags,
/// `And` requires all of them.
#[uniffi::remote(Enum)]
pub enum TagFilter {
    Or(Vec<String>),
    And(Vec<String>),
}

/// Home-screen snapshot: the most recently modified entries and one count
/// per media kind.
#[uniffi::remote(Record)]
pub struct Dashboard {
    pub recent: Vec<LibraryItem>,
    pub media_counts: HashMap<MediaKind, u32>,
}

/// A partial update to a library entry.
///
/// The outer `Option` distinguishes "set or clear" from "leave unchanged":
///
/// - `None` — leave the current value as-is;
/// - `Some(None)` — clear the field;
/// - `Some(Some(v))` — set it to `v`.
///
/// `playtime` only applies to `Game` entries.
#[uniffi::remote(Record)]
pub struct UpdateEntry {
    pub status: Option<Status>,
    pub notes: Option<Option<String>>,
    pub rating: Option<Option<u8>>,
    pub playtime: Option<Option<Duration>>,
}

/// A partial update to a collection: optionally rename it and/or apply a
/// list of member actions.
#[uniffi::remote(Record)]
pub struct UpdateCollection {
    pub title: Option<String>,
    pub update_entries: Vec<CollectionAction>,
}

/// Adds or removes a member entry from a collection.
#[uniffi::remote(Enum)]
pub enum CollectionAction {
    Add(i64),
    Remove(i64),
}
