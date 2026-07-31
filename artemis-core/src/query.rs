use std::collections::HashMap;
use std::time::Duration;

use crate::media::LibraryItem;
use crate::media::MediaKind;
use crate::media::Status;

#[derive(Default)]
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

#[derive(Default)]
pub struct SearchQuery {
    pub query: String,
    pub kind: Option<MediaKind>,
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
    Or(Vec<String>),
    And(Vec<String>),
}

pub struct Dashboard {
    pub recent: Vec<LibraryItem>,
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

#[derive(Default)]
pub struct UpdateCollection {
    pub title: Option<String>,
    pub update_entries: Vec<CollectionAction>,
}

pub enum CollectionAction {
    Add(i64),
    Remove(i64),
}
