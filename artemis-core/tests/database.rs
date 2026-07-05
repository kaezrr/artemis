//! CRUD tests for `Database::{open, add, get, update, delete}`.
//!
//! Coverage:
//! - `open` against a fresh in-memory database
//! - `add`: defaults, round-tripping every `MediaKind` variant, metadata/tags
//! - `get`: success, missing id, isolation between multiple entries
//! - `update`: status changes, the `Option<Option<T>>` "leave alone / set /
//!   clear" semantics on `notes`/`rating`/`playtime`, no-op updates, missing id,
//!   and `updated_at` advancing while `created_at` stays fixed
//! - `delete`: success, missing id, double-delete, no effect on other rows

mod common;

use artemis::media::Media;
use artemis::media::Status;
use artemis::query::UpdateEntry;
use common::anime;
use common::assert_not_found;
use common::game;
use common::mins;
use common::movie;
use common::tag;
use common::tags;
use common::test_db;
use common::tv_show;

#[tokio::test]
async fn open_creates_an_empty_database() {
    let db = test_db().await;
    assert_not_found(db.get(1).await, 1);
}

#[tokio::test]
async fn add_returns_entry_with_expected_defaults() {
    let db = test_db().await;
    let entry = db
        .add(anime(1, "Fullmetal Alchemist", "Bones", 64))
        .await
        .unwrap();

    assert!(entry.id > 0);
    assert_eq!(entry.metadata.title, "Fullmetal Alchemist");
    assert_eq!(entry.metadata.provider_id, 1);
    assert!(
        matches!(entry.status, Status::Planned),
        "new entries should default to Planned"
    );
    assert_eq!(entry.rating, None);
    assert_eq!(entry.notes, None);
    assert_eq!(
        entry.created_at, entry.updated_at,
        "a brand new entry should have identical created/updated timestamps"
    );

    match entry.media {
        Media::Anime { studio, episodes } => {
            assert_eq!(studio, Some("Bones".to_string()));
            assert_eq!(episodes, Some(64));
        }
        other => panic!("expected Media::Anime, got {other:?}"),
    }
}

#[tokio::test]
async fn add_round_trips_every_media_kind() {
    let db = test_db().await;

    let a = db.add(anime(1, "Anime A", "Studio A", 12)).await.unwrap();
    match a.media {
        Media::Anime { studio, episodes } => {
            assert_eq!(studio, Some("Studio A".to_string()));
            assert_eq!(episodes, Some(12));
        }
        other => panic!("expected Media::Anime, got {other:?}"),
    }

    let m = db
        .add(movie(2, "Movie A", "Director A", mins(120)))
        .await
        .unwrap();
    match m.media {
        Media::Movie { director, duration } => {
            assert_eq!(director, Some("Director A".to_string()));
            assert_eq!(duration, Some(mins(120)));
        }
        other => panic!("expected Media::Movie, got {other:?}"),
    }

    let g_with_playtime = db
        .add(game(3, "Game A", "Dev A", Some(mins(600))))
        .await
        .unwrap();
    match g_with_playtime.media {
        Media::Game {
            developer,
            playtime,
        } => {
            assert_eq!(developer, Some("Dev A".to_string()));
            assert_eq!(playtime, Some(mins(600)));
        }
        other => panic!("expected Media::Game, got {other:?}"),
    }

    let g_without_playtime = db.add(game(4, "Game B", "Dev B", None)).await.unwrap();
    match g_without_playtime.media {
        Media::Game { playtime, .. } => assert_eq!(playtime, None),
        other => panic!("expected Media::Game, got {other:?}"),
    }

    let t = db
        .add(tv_show(5, "Show A", "Director B", 24))
        .await
        .unwrap();
    match t.media {
        Media::TVShow { creator, episodes } => {
            assert_eq!(creator, Some("Director B".to_string()));
            assert_eq!(episodes, Some(24));
        }
        other => panic!("expected Media::TVShow, got {other:?}"),
    }
}

#[tokio::test]
async fn add_round_trips_metadata_and_tags() {
    let db = test_db().await;
    let mut sr = anime(1, "Cowboy Bebop", "Sunrise", 26);
    sr.metadata.tags = tags(&["action", "space-western"]);
    sr.metadata.description = Some("A ragtag crew of bounty hunters.".to_string());
    sr.metadata.wide_url = Some("https://example.test/wide.jpg".to_string());
    sr.metadata.release_year = Some(1998);

    let entry = db.add(sr).await.unwrap();

    assert_eq!(
        entry.metadata.description,
        Some("A ragtag crew of bounty hunters.".to_string())
    );
    assert_eq!(
        entry.metadata.wide_url.as_deref(),
        Some("https://example.test/wide.jpg")
    );
    assert_eq!(entry.metadata.release_year, Some(1998));
    assert_eq!(entry.metadata.tags.len(), 2);
    assert!(entry.metadata.tags.contains(&tag("action")));
    assert!(entry.metadata.tags.contains(&tag("space-western")));
}

#[tokio::test]
async fn get_fetches_a_previously_added_entry() {
    let db = test_db().await;
    let added = db
        .add(movie(1, "Arrival", "Denis Villeneuve", mins(116)))
        .await
        .unwrap();

    let fetched = db.get(added.id).await.unwrap();
    assert_eq!(fetched.id, added.id);
    assert_eq!(fetched.metadata.title, "Arrival");
}

#[tokio::test]
async fn get_nonexistent_id_errors() {
    let db = test_db().await;
    assert_not_found(db.get(9999).await, 9999);
}

#[tokio::test]
async fn entries_have_independent_ids_and_data() {
    let db = test_db().await;
    let a = db.add(anime(1, "A", "Studio", 1)).await.unwrap();
    let b = db.add(anime(2, "B", "Studio", 1)).await.unwrap();
    let c = db.add(anime(3, "C", "Studio", 1)).await.unwrap();

    assert_ne!(a.id, b.id);
    assert_ne!(b.id, c.id);
    assert_ne!(a.id, c.id);

    assert_eq!(db.get(a.id).await.unwrap().metadata.title, "A");
    assert_eq!(db.get(b.id).await.unwrap().metadata.title, "B");
    assert_eq!(db.get(c.id).await.unwrap().metadata.title, "C");
}

#[tokio::test]
async fn update_status_persists() {
    let db = test_db().await;
    let added = db.add(anime(1, "A", "Studio", 12)).await.unwrap();
    assert!(matches!(added.status, Status::Planned));

    let updated = db
        .update(
            added.id,
            UpdateEntry {
                status: Some(Status::InProgress),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert!(matches!(updated.status, Status::InProgress));

    let refetched = db.get(added.id).await.unwrap();
    assert!(matches!(refetched.status, Status::InProgress));
}

#[tokio::test]
async fn update_rating_option_option_semantics() {
    let db = test_db().await;
    let added = db.add(anime(1, "A", "Studio", 12)).await.unwrap();
    assert_eq!(added.rating, None);

    // Some(Some(x)) sets the rating.
    let updated = db
        .update(
            added.id,
            UpdateEntry {
                rating: Some(Some(6)),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(updated.rating, Some(6));

    // Field left as `None` means "don't touch it" -- an unrelated update must
    // not clear the rating.
    let updated = db
        .update(
            added.id,
            UpdateEntry {
                notes: Some(Some("watched season 1".to_string())),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(updated.rating, Some(6));

    // Some(None) explicitly clears the rating.
    let updated = db
        .update(
            added.id,
            UpdateEntry {
                rating: Some(None),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(updated.rating, None);
}

#[tokio::test]
async fn update_notes_option_option_semantics() {
    let db = test_db().await;
    let added = db.add(anime(1, "A", "Studio", 12)).await.unwrap();
    assert_eq!(added.notes, None);

    let updated = db
        .update(
            added.id,
            UpdateEntry {
                notes: Some(Some("great show".to_string())),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(updated.notes.as_deref(), Some("great show"));

    // Leaving it `None` preserves the existing note.
    let updated = db
        .update(
            added.id,
            UpdateEntry {
                rating: Some(Some(5)),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(updated.notes.as_deref(), Some("great show"));

    let updated = db
        .update(
            added.id,
            UpdateEntry {
                notes: Some(None),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(updated.notes, None);
}

#[tokio::test]
async fn update_playtime_for_a_game_entry() {
    let db = test_db().await;
    let added = db
        .add(game(1, "Elden Ring", "FromSoftware", None))
        .await
        .unwrap();

    let updated = db
        .update(
            added.id,
            UpdateEntry {
                playtime: Some(Some(mins(6000))),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    match updated.media {
        Media::Game { playtime, .. } => assert_eq!(playtime, Some(mins(6000))),
        other => panic!("expected Media::Game, got {other:?}"),
    }

    let updated = db
        .update(
            added.id,
            UpdateEntry {
                playtime: Some(None),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    match updated.media {
        Media::Game { playtime, .. } => assert_eq!(playtime, None),
        other => panic!("expected Media::Game, got {other:?}"),
    }
}

#[tokio::test]
async fn update_with_all_fields_none_is_a_noop() {
    let db = test_db().await;
    let added = db.add(anime(1, "A", "Studio", 12)).await.unwrap();

    let updated = db.update(added.id, UpdateEntry::default()).await.unwrap();

    assert!(matches!(updated.status, Status::Planned));
    assert_eq!(updated.rating, None);
    assert_eq!(updated.notes, None);
}

#[tokio::test]
async fn update_nonexistent_id_errors() {
    let db = test_db().await;
    assert_not_found(db.update(9999, UpdateEntry::default()).await, 9999);
}

#[tokio::test]
async fn update_advances_updated_at_but_not_created_at() {
    let db = test_db().await;
    let added = db.add(anime(1, "A", "Studio", 12)).await.unwrap();
    assert_eq!(added.created_at, added.updated_at);

    // Give timestamp storage a real, measurable gap to detect. If your
    // `UtcDateTime` has sub-second precision this can be shortened.
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;

    let updated = db
        .update(
            added.id,
            UpdateEntry {
                status: Some(Status::Finished),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    assert_eq!(updated.created_at, added.created_at);
    assert!(updated.updated_at > added.created_at);
}

#[tokio::test]
async fn delete_removes_the_entry() {
    let db = test_db().await;
    let added = db.add(anime(1, "A", "Studio", 12)).await.unwrap();
    db.delete(added.id).await.unwrap();
    assert_not_found(db.get(added.id).await, added.id);
}

#[tokio::test]
async fn delete_nonexistent_id_errors() {
    let db = test_db().await;
    assert_not_found(db.delete(9999).await, 9999);
}

#[tokio::test]
async fn deleting_twice_errors_on_the_second_call() {
    let db = test_db().await;
    let added = db.add(anime(1, "A", "Studio", 12)).await.unwrap();
    db.delete(added.id).await.unwrap();
    assert_not_found(db.delete(added.id).await, added.id);
}

#[tokio::test]
async fn delete_does_not_affect_other_entries() {
    let db = test_db().await;
    let a = db.add(anime(1, "A", "Studio", 12)).await.unwrap();
    let b = db.add(anime(2, "B", "Studio", 12)).await.unwrap();

    db.delete(a.id).await.unwrap();

    assert_not_found(db.get(a.id).await, a.id);
    assert!(db.get(b.id).await.is_ok());
}
