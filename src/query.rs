use std::collections::HashMap;

use crate::media::Duration;
use crate::media::LibraryEntry;
use crate::media::MediaKind;
use crate::media::Status;
use crate::media::Tag;

#[derive(Default)]
pub struct LibraryQuery {
    pub search: Option<String>,
    pub kind: Option<Vec<MediaKind>>,

    pub sort_by: SortBy,
    pub order: SortOrder,

    pub status: Option<Status>,
    pub tag_filter: Option<TagFilter>,

    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Default)]
pub struct SearchQuery {
    pub query: String,
    pub kind: Option<MediaKind>,
    pub in_library: bool,
}

#[derive(Default)]
pub enum SortOrder {
    #[default]
    Ascending,
    Descending,
}

#[derive(Default)]
pub enum SortBy {
    #[default]
    Title,
    Rating,
    ReleaseYear,
    LastModified,
}

pub enum TagFilter {
    Or(Vec<Tag>),
    And(Vec<Tag>),
}

pub struct Dashboard {
    pub recent: Vec<LibraryEntry>,
    pub media_counts: HashMap<MediaKind, u32>,
}

/// Used to update a library entry
#[derive(Default)]
pub struct UpdateEntry {
    pub status: Option<Status>,
    pub notes: Option<Option<String>>,
    pub rating: Option<Option<u8>>,
    pub playtime: Option<Option<Duration>>,
}
