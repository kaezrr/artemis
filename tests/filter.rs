//! Tests for `Database::query` / `LibraryQuery`.
//!
//! Uses the fixed, varied fixture set from `common::seed_library` (8 entries:
//! every `MediaKind`, a spread of release years, overlapping/non-overlapping
//! tags, alphabetically distinct titles, and a mix of statuses/ratings
//! including `None` ratings).
//!
//! `collection_id` is intentionally left untested here (always `None` via
//! `..Default::default()`) — it's covered thoroughly in `collection.rs`
//! alongside collections in general.
//!
//! Coverage:
//! - default query returns everything
//! - `search`: substring match, case-insensitivity, no-match case
//! - `kind` filter
//! - `status` filter
//! - filters combined
//! - `sort_by` for every `SortBy` variant, both orders where meaningful
//! - `tag_filter`: `Or` and `And`
//! - `limit` / `offset`, individually and combined

mod common;

use std::collections::HashMap;
use std::collections::HashSet;

use artemis::media::MediaKind;
use artemis::media::Status;
use artemis::query::LibraryQuery;
use artemis::query::SortBy;
use artemis::query::SortOrder;
use artemis::query::TagFilter;
use common::*;

#[tokio::test]
async fn default_query_returns_every_entry() {
    let db = test_db().await;
    seed_library(&db).await;

    let items = db.query(LibraryQuery::default()).await.unwrap();
    assert_eq!(items.len(), 8);
}

#[tokio::test]
async fn search_filters_by_title_substring() {
    let db = test_db().await;
    seed_library(&db).await;

    let items = db
        .query(LibraryQuery {
            search: Some("the".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();

    let titles: HashSet<_> = items.iter().map(|i| i.title.clone()).collect();
    assert_eq!(items.len(), 2);
    assert!(titles.contains("The Room"));
    assert!(titles.contains("The Office"));
}

#[tokio::test]
async fn search_is_case_insensitive() {
    let db = test_db().await;
    seed_library(&db).await;

    let items = db
        .query(LibraryQuery {
            search: Some("HADES".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title, "Hades");
}

#[tokio::test]
async fn search_with_no_matches_returns_empty() {
    let db = test_db().await;
    seed_library(&db).await;

    let items = db
        .query(LibraryQuery {
            search: Some("this title does not exist".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();

    assert!(items.is_empty());
}

#[tokio::test]
async fn kind_filter_returns_only_matching_kind() {
    let db = test_db().await;
    seed_library(&db).await;

    let items = db
        .query(LibraryQuery {
            kind: Some(MediaKind::Game),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(items.len(), 2);
    assert!(items.iter().all(|i| matches!(i.kind, MediaKind::Game)));
}

#[tokio::test]
async fn status_filter_returns_only_matching_status() {
    let db = test_db().await;
    seed_library(&db).await;

    let items = db
        .query(LibraryQuery {
            status: Some(Status::Finished),
            ..Default::default()
        })
        .await
        .unwrap();

    // Attack on Titan, K-On!, Breaking Bad are seeded as Finished.
    assert_eq!(items.len(), 3);
    assert!(items.iter().all(|i| matches!(i.status, Status::Finished)));
}

#[tokio::test]
async fn combined_kind_and_status_filter() {
    let db = test_db().await;
    seed_library(&db).await;

    let items = db
        .query(LibraryQuery {
            kind: Some(MediaKind::Anime),
            status: Some(Status::Finished),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(items.len(), 2);
    assert_eq!(items[0].title, "Attack on Titan");
    assert_eq!(items[1].title, "K-On!");
}

#[tokio::test]
async fn combined_search_and_status_filter() {
    let db = test_db().await;
    seed_library(&db).await;

    let items = db
        .query(LibraryQuery {
            search: Some("the".to_string()),
            status: Some(Status::Planned),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title, "The Office");
}

#[tokio::test]
async fn sort_by_title_ascending() {
    let db = test_db().await;
    seed_library(&db).await;

    let items = db
        .query(LibraryQuery {
            sort_by: SortBy::Title,
            order: SortOrder::Ascending,
            ..Default::default()
        })
        .await
        .unwrap();

    let titles: Vec<_> = items.iter().map(|i| i.title.clone()).collect();
    let mut expected = titles.clone();
    expected.sort();
    assert_eq!(titles, expected);
}

#[tokio::test]
async fn sort_by_title_descending() {
    let db = test_db().await;
    seed_library(&db).await;

    let items = db
        .query(LibraryQuery {
            sort_by: SortBy::Title,
            order: SortOrder::Descending,
            ..Default::default()
        })
        .await
        .unwrap();

    let titles: Vec<_> = items.iter().map(|i| i.title.clone()).collect();
    let mut expected = titles.clone();
    expected.sort();
    expected.reverse();
    assert_eq!(titles, expected);
}

#[tokio::test]
async fn sort_by_rating_orders_rated_items_relative_to_each_other() {
    let db = test_db().await;
    seed_library(&db).await;

    let items = db
        .query(LibraryQuery {
            sort_by: SortBy::Rating,
            order: SortOrder::Descending,
            ..Default::default()
        })
        .await
        .unwrap();

    // Only assert on the relative order of rated items -- where `None`
    // ratings land is an implementation choice we don't want to over-specify.
    let rated: Vec<u8> = items.iter().filter_map(|i| i.rating).collect();
    let mut expected = rated.clone();
    expected.sort_unstable_by(|a, b| b.cmp(a));
    assert_eq!(rated, expected);
}

#[tokio::test]
async fn sort_by_release_year_ascending() {
    let db = test_db().await;
    seed_library(&db).await;

    let items = db
        .query(LibraryQuery {
            sort_by: SortBy::ReleaseYear,
            order: SortOrder::Ascending,
            ..Default::default()
        })
        .await
        .unwrap();

    // release_year isn't exposed on LibraryItem, so we cross-reference by
    // title against the years seed_library assigned.
    let years: HashMap<&str, u32> = [
        ("Attack on Titan", 2013),
        ("K-On!", 2009),
        ("Parasite", 2019),
        ("The Room", 2003),
        ("Hades", 2020),
        ("Stardew Valley", 2016),
        ("Breaking Bad", 2008),
        ("The Office", 2005),
    ]
    .into_iter()
    .collect();

    let observed: Vec<u32> = items.iter().map(|i| years[i.title.as_str()]).collect();
    let mut expected = observed.clone();
    expected.sort();
    assert_eq!(observed, expected);
}

#[tokio::test]
async fn sort_by_last_modified_puts_the_most_recently_updated_first() {
    use artemis::query::UpdateEntry;

    let db = test_db().await;
    let entries = seed_library(&db).await;
    let target = entries
        .iter()
        .find(|e| e.metadata.title == "K-On!")
        .unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
    db.update(
        target.id,
        UpdateEntry {
            status: Some(Status::Finished),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let items = db
        .query(LibraryQuery {
            sort_by: SortBy::LastModified,
            order: SortOrder::Descending,
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(items[0].id, target.id);
}

#[tokio::test]
async fn tag_filter_or_matches_any_of_the_given_tags() {
    let db = test_db().await;
    seed_library(&db).await;

    let items = db
        .query(LibraryQuery {
            tag_filter: Some(TagFilter::Or(tags(&["comedy", "roguelike"]))),
            ..Default::default()
        })
        .await
        .unwrap();

    let titles: HashSet<_> = items.iter().map(|i| i.title.clone()).collect();
    let expected: HashSet<_> = ["K-On!", "The Room", "The Office", "Hades"]
        .into_iter()
        .map(String::from)
        .collect();
    assert_eq!(titles, expected);
}

#[tokio::test]
async fn tag_filter_and_requires_every_tag_to_be_present() {
    let db = test_db().await;
    seed_library(&db).await;

    let items = db
        .query(LibraryQuery {
            tag_filter: Some(TagFilter::And(tags(&["drama", "thriller"]))),
            ..Default::default()
        })
        .await
        .unwrap();

    let titles: HashSet<_> = items.iter().map(|i| i.title.clone()).collect();
    let expected: HashSet<_> = ["Parasite", "Breaking Bad"]
        .into_iter()
        .map(String::from)
        .collect();
    assert_eq!(titles, expected);
}

#[tokio::test]
async fn tag_filter_with_no_matches_returns_empty() {
    let db = test_db().await;
    seed_library(&db).await;

    let items = db
        .query(LibraryQuery {
            tag_filter: Some(TagFilter::And(tags(&["action", "comedy"]))),
            ..Default::default()
        })
        .await
        .unwrap();

    assert!(items.is_empty());
}

#[tokio::test]
async fn limit_restricts_the_result_count() {
    let db = test_db().await;
    seed_library(&db).await;

    let items = db
        .query(LibraryQuery {
            sort_by: SortBy::Title,
            limit: Some(3),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(items.len(), 3);
}

#[tokio::test]
async fn offset_skips_leading_results() {
    let db = test_db().await;
    seed_library(&db).await;

    let all = db
        .query(LibraryQuery {
            sort_by: SortBy::Title,
            ..Default::default()
        })
        .await
        .unwrap();

    let offset_items = db
        .query(LibraryQuery {
            sort_by: SortBy::Title,
            offset: Some(2),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(offset_items.len(), all.len() - 2);
    assert_eq!(offset_items[0].title, all[2].title);
}

#[tokio::test]
async fn limit_and_offset_paginate_together() {
    let db = test_db().await;
    seed_library(&db).await;

    let all = db
        .query(LibraryQuery {
            sort_by: SortBy::Title,
            ..Default::default()
        })
        .await
        .unwrap();

    let page = db
        .query(LibraryQuery {
            sort_by: SortBy::Title,
            limit: Some(2),
            offset: Some(2),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(page.len(), 2);
    assert_eq!(page[0].title, all[2].title);
    assert_eq!(page[1].title, all[3].title);
}

#[tokio::test]
async fn offset_past_the_end_returns_empty() {
    let db = test_db().await;
    seed_library(&db).await;

    let items = db
        .query(LibraryQuery {
            offset: Some(100),
            ..Default::default()
        })
        .await
        .unwrap();

    assert!(items.is_empty());
}
