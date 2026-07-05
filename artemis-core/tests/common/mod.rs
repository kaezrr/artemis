//! Shared test utilities for the artemis integration test suite.
//!
//! ## Remaining assumptions about APIs not shown in the provided code
//!
//! 1. `artemis::Database` is exported from the crate root (`use artemis::Database;`).
//! 2. `Database::open("sqlite::memory:")` runs migrations itself — no separate
//!    `migrate()`/`setup()` call is required before the schema is usable.
//! 3. `artemis::Error::NotFound(i64)` is used not just for missing library
//!    entries but also for missing collections, since it's the only variant
//!    that fits "no row with this id" and the id type matches. If collections
//!    use a different error path, the `assert_not_found` calls in
//!    `collection.rs` are the ones to revisit.
//!
//! If the suite doesn't compile, these are the first things to check.
#![allow(unused)]

use artemis::Database;
use artemis::media::Duration;
use artemis::media::Media;
use artemis::media::ProviderMetadata;
use artemis::media::SearchResult;
use artemis::media::Status;
use artemis::query::UpdateEntry;

/// Opens a fresh, isolated in-memory database. Each test gets its own instance,
/// so tests never see each other's data.
pub async fn test_db() -> Database {
    Database::open("sqlite::memory:")
        .await
        .expect("failed to open in-memory test database")
}

pub fn tag(s: &str) -> String {
    s.to_string()
}

pub fn tags(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| tag(s)).collect()
}

/// Builds a `Duration` from a number of minutes (the type itself only exposes
/// a `seconds` constructor).
pub fn mins(m: i64) -> Duration {
    Duration::seconds(m * 60)
}

/// Asserts a `Result` failed with `Error::NotFound(id)` specifically, rather
/// than just "some error". Panics with a useful message otherwise.
pub fn assert_not_found<T: std::fmt::Debug>(result: Result<T, artemis::Error>, id: i64) {
    match result {
        Err(artemis::Error::NotFound(found_id)) => {
            assert_eq!(found_id, id, "NotFound error carried the wrong id")
        }
        Err(other) => panic!("expected Error::NotFound({id}), got a different error: {other:?}"),
        Ok(value) => panic!("expected Error::NotFound({id}), got Ok({value:?})"),
    }
}

/// Sensible default metadata; tests override only the fields they care about
/// via the `..metadata(..)` struct update syntax if needed.
pub fn metadata(provider_id: i64, title: &str) -> ProviderMetadata {
    ProviderMetadata {
        provider: "test-provider".to_string(),
        provider_id,
        title: title.to_string(),
        cover_url: format!("https://example.test/cover/{provider_id}.jpg"),
        wide_url: None,
        description: Some(format!("Description for {title}")),
        tags: Vec::new(),
        release_year: Some(2020),
    }
}

pub fn anime(provider_id: i64, title: &str, studio: &str, episodes: u32) -> SearchResult {
    SearchResult {
        media: Media::Anime {
            studio: Some(studio.to_string()),
            episodes: Some(episodes),
        },
        metadata: metadata(provider_id, title),
        in_library: false,
    }
}

pub fn movie(provider_id: i64, title: &str, director: &str, duration: Duration) -> SearchResult {
    SearchResult {
        media: Media::Movie {
            director: Some(director.to_string()),
            duration: Some(duration),
        },
        metadata: metadata(provider_id, title),
        in_library: false,
    }
}

pub fn game(
    provider_id: i64,
    title: &str,
    developer: &str,
    playtime: Option<Duration>,
) -> SearchResult {
    SearchResult {
        media: Media::Game {
            developer: Some(developer.to_string()),
            playtime,
        },
        metadata: metadata(provider_id, title),
        in_library: false,
    }
}

pub fn tv_show(provider_id: i64, title: &str, director: &str, episodes: u32) -> SearchResult {
    SearchResult {
        media: Media::TVShow {
            director: Some(director.to_string()),
            episodes: Some(episodes),
        },
        metadata: metadata(provider_id, title),
        in_library: false,
    }
}

/// Attaches tags + a release year to a `SearchResult` in one step. Only used by
/// `seed_library` below, but pulled out for readability.
fn with(mut sr: SearchResult, tag_list: &[&str], year: u32) -> SearchResult {
    sr.metadata.tags = tags(tag_list);
    sr.metadata.release_year = Some(year);
    sr
}

/// Seeds a deliberately varied library: every `MediaKind`, a spread of years
/// (for `SortBy::ReleaseYear`), overlapping/non-overlapping tags (for
/// `TagFilter::Or`/`And`), titles that sort predictably (for `SortBy::Title`),
/// and a mix of statuses + ratings (including `None` ratings) applied via
/// `update` after insert, since `add` never sets those.
///
/// Used by `filter.rs`. Returns the entries in insertion order so tests can
/// index into them by position when they need a specific title's id.
pub async fn seed_library(db: &Database) -> Vec<artemis::media::LibraryEntry> {
    let inserts = vec![
        db.add(with(
            anime(1, "Attack on Titan", "WIT Studio", 25),
            &["action", "drama"],
            2013,
        ))
        .await,
        db.add(with(
            anime(2, "K-On!", "Kyoto Animation", 12),
            &["slice-of-life", "comedy"],
            2009,
        ))
        .await,
        db.add(with(
            movie(3, "Parasite", "Bong Joon-ho", mins(132)),
            &["drama", "thriller"],
            2019,
        ))
        .await,
        db.add(with(
            movie(4, "The Room", "Tommy Wiseau", mins(99)),
            &["comedy"],
            2003,
        ))
        .await,
        db.add(with(
            game(5, "Hades", "Supergiant", Some(mins(4000))),
            &["action", "roguelike"],
            2020,
        ))
        .await,
        db.add(with(
            game(6, "Stardew Valley", "ConcernedApe", None),
            &["slice-of-life"],
            2016,
        ))
        .await,
        db.add(with(
            tv_show(7, "Breaking Bad", "Vince Gilligan", 62),
            &["drama", "thriller"],
            2008,
        ))
        .await,
        db.add(with(
            tv_show(8, "The Office", "Greg Daniels", 201),
            &["comedy"],
            2005,
        ))
        .await,
    ];

    let entries: Vec<_> = inserts
        .into_iter()
        .map(|r| r.expect("seed insert should succeed"))
        .collect();

    // (status, rating) per entry above, in the same order, giving a mix of
    // every status and both `Some`/`None` ratings.
    let statuses_and_ratings: [(Status, Option<u8>); 8] = [
        (Status::Finished, Some(6)),   // Attack on Titan
        (Status::Finished, Some(5)),   // K-On!
        (Status::InProgress, Some(6)), // Parasite
        (Status::Dropped, Some(1)),    // The Room
        (Status::InProgress, Some(7)), // Hades
        (Status::OnHold, None),        // Stardew Valley
        (Status::Finished, Some(6)),   // Breaking Bad
        (Status::Planned, None),       // The Office
    ];

    let mut updated = Vec::with_capacity(entries.len());
    for (entry, (status, rating)) in entries.into_iter().zip(statuses_and_ratings) {
        let e = db
            .update(
                entry.id,
                UpdateEntry {
                    status: Some(status),
                    rating: Some(rating),
                    ..Default::default()
                },
            )
            .await
            .expect("seed status/rating update should succeed");
        updated.push(e);
    }
    updated
}
