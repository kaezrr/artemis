//! Tests for `Database::{add_collection, get_collection, get_collections,
//! update_collection, delete_collection}`, plus `LibraryQuery.collection_id`
//! (deliberately skipped in `filter.rs` in favor of thorough coverage here).
//!
//! Coverage:
//! - `add_collection`: correct `count`, empty collections
//! - `get_collection`: success, missing id
//! - `get_collections`: returns all, empty database case
//! - `update_collection`: title rename, `CollectionAction::Add`/`Remove`
//!   individually and combined, missing id
//! - `delete_collection`: removes the collection, leaves underlying media
//!   untouched, missing id
//! - `collection_id` filter: membership-only results, combined with other
//!   `LibraryQuery` filters, empty collection

mod common;

use std::collections::HashSet;

use artemis::media::MediaKind;
use artemis::query::CollectionAction;
use artemis::query::LibraryQuery;
use artemis::query::UpdateCollection;
use common::*;

#[tokio::test]
async fn add_collection_creates_with_correct_count() {
    let db = test_db().await;
    let a = db.add(anime(1, "A", "Studio", 12)).await.unwrap();
    let b = db.add(anime(2, "B", "Studio", 12)).await.unwrap();

    let collection = db.add_collection("Favorites", &[a.id, b.id]).await.unwrap();

    assert!(collection.id > 0);
    assert_eq!(collection.title, "Favorites");
    assert_eq!(collection.count, 2);
}

#[tokio::test]
async fn add_collection_with_no_media_has_zero_count() {
    let db = test_db().await;
    let collection = db.add_collection("Empty", &[]).await.unwrap();
    assert_eq!(collection.count, 0);
}

#[tokio::test]
async fn get_collection_fetches_by_id() {
    let db = test_db().await;
    let a = db.add(anime(1, "A", "Studio", 12)).await.unwrap();
    let created = db.add_collection("Favorites", &[a.id]).await.unwrap();

    let fetched = db.get_collection(created.id).await.unwrap();
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.title, "Favorites");
    assert_eq!(fetched.count, 1);
}

#[tokio::test]
async fn get_collection_nonexistent_id_errors() {
    let db = test_db().await;
    assert_not_found(db.get_collection(9999).await, 9999);
}

#[tokio::test]
async fn get_collections_returns_all_of_them() {
    let db = test_db().await;
    let a = db.add(anime(1, "A", "Studio", 12)).await.unwrap();
    db.add_collection("Favorites", &[a.id]).await.unwrap();
    db.add_collection("Watchlist", &[]).await.unwrap();
    db.add_collection("Backlog", &[]).await.unwrap();

    let collections = db.get_collections().await.unwrap();
    assert_eq!(collections.len(), 3);

    let titles: HashSet<_> = collections.iter().map(|c| c.title.clone()).collect();
    let expected: HashSet<_> = ["Favorites", "Watchlist", "Backlog"]
        .into_iter()
        .map(String::from)
        .collect();
    assert_eq!(titles, expected);
}

#[tokio::test]
async fn get_collections_on_empty_database_is_empty() {
    let db = test_db().await;
    assert!(db.get_collections().await.unwrap().is_empty());
}

#[tokio::test]
async fn update_collection_renames_title() {
    let db = test_db().await;
    let created = db.add_collection("Old Name", &[]).await.unwrap();

    let updated = db
        .update_collection(
            created.id,
            UpdateCollection {
                title: Some("New Name".to_string()),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    assert_eq!(updated.title, "New Name");
    assert_eq!(
        db.get_collection(created.id).await.unwrap().title,
        "New Name"
    );
}

#[tokio::test]
async fn update_collection_add_increases_count() {
    let db = test_db().await;
    let a = db.add(anime(1, "A", "Studio", 12)).await.unwrap();
    let b = db.add(anime(2, "B", "Studio", 12)).await.unwrap();
    let created = db.add_collection("Favorites", &[a.id]).await.unwrap();
    assert_eq!(created.count, 1);

    let updated = db
        .update_collection(
            created.id,
            UpdateCollection {
                update_entries: vec![CollectionAction::Add(b.id)],
                ..Default::default()
            },
        )
        .await
        .unwrap();

    assert_eq!(updated.count, 2);
}

#[tokio::test]
async fn update_collection_remove_decreases_count() {
    let db = test_db().await;
    let a = db.add(anime(1, "A", "Studio", 12)).await.unwrap();
    let b = db.add(anime(2, "B", "Studio", 12)).await.unwrap();
    let created = db.add_collection("Favorites", &[a.id, b.id]).await.unwrap();
    assert_eq!(created.count, 2);

    let updated = db
        .update_collection(
            created.id,
            UpdateCollection {
                update_entries: vec![CollectionAction::Remove(a.id)],
                ..Default::default()
            },
        )
        .await
        .unwrap();

    assert_eq!(updated.count, 1);
}

#[tokio::test]
async fn update_collection_add_and_remove_together() {
    let db = test_db().await;
    let a = db.add(anime(1, "A", "Studio", 12)).await.unwrap();
    let b = db.add(anime(2, "B", "Studio", 12)).await.unwrap();
    let c = db.add(anime(3, "C", "Studio", 12)).await.unwrap();
    let created = db.add_collection("Favorites", &[a.id, b.id]).await.unwrap();

    let updated = db
        .update_collection(
            created.id,
            UpdateCollection {
                update_entries: vec![CollectionAction::Remove(a.id), CollectionAction::Add(c.id)],
                ..Default::default()
            },
        )
        .await
        .unwrap();

    assert_eq!(updated.count, 2);

    let items = db
        .query(LibraryQuery {
            collection_id: Some(created.id),
            ..Default::default()
        })
        .await
        .unwrap();
    let ids: HashSet<_> = items.iter().map(|i| i.id).collect();
    assert_eq!(ids, [b.id, c.id].into_iter().collect());
}

#[tokio::test]
async fn update_collection_title_and_membership_together() {
    let db = test_db().await;
    let a = db.add(anime(1, "A", "Studio", 12)).await.unwrap();
    let b = db.add(anime(2, "B", "Studio", 12)).await.unwrap();
    let created = db.add_collection("Old Name", &[a.id]).await.unwrap();

    let updated = db
        .update_collection(
            created.id,
            UpdateCollection {
                title: Some("New Name".to_string()),
                update_entries: vec![CollectionAction::Add(b.id)],
            },
        )
        .await
        .unwrap();

    assert_eq!(updated.title, "New Name");
    assert_eq!(updated.count, 2);
}

#[tokio::test]
async fn update_collection_nonexistent_id_errors() {
    let db = test_db().await;
    assert_not_found(
        db.update_collection(9999, UpdateCollection::default())
            .await,
        9999,
    );
}

#[tokio::test]
async fn delete_collection_removes_it() {
    let db = test_db().await;
    let created = db.add_collection("Temp", &[]).await.unwrap();
    db.delete_collection(created.id).await.unwrap();
    assert_not_found(db.get_collection(created.id).await, created.id);
}

#[tokio::test]
async fn delete_collection_does_not_delete_underlying_media() {
    let db = test_db().await;
    let a = db.add(anime(1, "A", "Studio", 12)).await.unwrap();
    let created = db.add_collection("Temp", &[a.id]).await.unwrap();

    db.delete_collection(created.id).await.unwrap();

    assert!(db.get(a.id).await.is_ok());
}

#[tokio::test]
async fn delete_collection_nonexistent_id_errors() {
    let db = test_db().await;
    assert_not_found(db.delete_collection(9999).await, 9999);
}

#[tokio::test]
async fn delete_collection_does_not_affect_other_collections() {
    let db = test_db().await;
    let keep = db.add_collection("Keep", &[]).await.unwrap();
    let remove = db.add_collection("Remove", &[]).await.unwrap();

    db.delete_collection(remove.id).await.unwrap();

    assert!(db.get_collection(keep.id).await.is_ok());
    assert_not_found(db.get_collection(remove.id).await, remove.id);
}

#[tokio::test]
async fn collection_id_filter_returns_only_members() {
    let db = test_db().await;
    let a = db.add(anime(1, "A", "Studio", 12)).await.unwrap();
    let b = db.add(anime(2, "B", "Studio", 12)).await.unwrap();
    let c = db.add(anime(3, "C", "Studio", 12)).await.unwrap();

    let collection = db.add_collection("Favorites", &[a.id, c.id]).await.unwrap();

    let items = db
        .query(LibraryQuery {
            collection_id: Some(collection.id),
            ..Default::default()
        })
        .await
        .unwrap();

    let ids: HashSet<_> = items.iter().map(|i| i.id).collect();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&a.id));
    assert!(ids.contains(&c.id));
    assert!(!ids.contains(&b.id));
}

#[tokio::test]
async fn collection_id_filter_combines_with_kind_filter() {
    let db = test_db().await;
    let a = db.add(anime(1, "Anime A", "Studio", 12)).await.unwrap();
    let m = db.add(movie(2, "Movie A", "Dir", mins(100))).await.unwrap();
    let collection = db.add_collection("Mixed", &[a.id, m.id]).await.unwrap();

    let items = db
        .query(LibraryQuery {
            collection_id: Some(collection.id),
            kind: Some(MediaKind::Movie),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].id, m.id);
}

#[tokio::test]
async fn collection_id_filter_combines_with_search_filter() {
    let db = test_db().await;
    let a = db
        .add(anime(1, "Cowboy Bebop", "Sunrise", 26))
        .await
        .unwrap();
    let b = db
        .add(anime(2, "Bebop Knockoff", "Other", 12))
        .await
        .unwrap();
    let c = db
        .add(anime(3, "Unrelated Show", "Other", 12))
        .await
        .unwrap();
    let collection = db
        .add_collection("Bebop Related", &[a.id, b.id, c.id])
        .await
        .unwrap();

    let items = db
        .query(LibraryQuery {
            collection_id: Some(collection.id),
            search: Some("bebop".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();

    let ids: HashSet<_> = items.iter().map(|i| i.id).collect();
    assert_eq!(ids, [a.id, b.id].into_iter().collect());
}

#[tokio::test]
async fn collection_id_filter_with_empty_collection_returns_empty() {
    let db = test_db().await;
    db.add(anime(1, "A", "Studio", 12)).await.unwrap();
    let collection = db.add_collection("Empty", &[]).await.unwrap();

    let items = db
        .query(LibraryQuery {
            collection_id: Some(collection.id),
            ..Default::default()
        })
        .await
        .unwrap();

    assert!(items.is_empty());
}

#[tokio::test]
async fn collection_id_filter_nonexistent_collection_returns_empty() {
    let db = test_db().await;
    db.add(anime(1, "A", "Studio", 12)).await.unwrap();

    let items = db
        .query(LibraryQuery {
            collection_id: Some(9999),
            ..Default::default()
        })
        .await
        .unwrap();

    assert!(items.is_empty());
}
