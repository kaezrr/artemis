use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use artemis::Database;
use artemis::Error;
use artemis::media::*;
use artemis::query::*;

// Global atomic counter to guarantee every test gets its own isolated database instance
static DB_COUNTER: AtomicUsize = AtomicUsize::new(0);

// --- Fixed Test Helpers ---

async fn setup_clean_db() -> std::result::Result<Database, Error> {
    let test_id = DB_COUNTER.fetch_add(1, Ordering::SeqCst);

    // By giving each connection a unique in-memory filename, SQLite keeps them
    // 100% isolated from other tests running in parallel.
    let unique_db_url = format!("sqlite:file:memdb_{test_id}?mode=memory&cache=shared");

    let db = Database::open(&unique_db_url).await?;

    // sqlx::migrate!("./migrations").run(&db.pool).await?;

    Ok(db)
}

// Added `provider_id` as an argument to prevent UNIQUE constraint violations
fn create_mock_search_result(media: Media, title: &str, provider_id: i64) -> SearchResult {
    SearchResult {
        media,
        metadata: ProviderMetadata {
            provider: "test_provider".to_string(),
            provider_id, // <--- Dynamic ID
            title: title.to_string(),
            cover_url: "https://example.com/cover.jpg".to_string(),
            wide_url: None,
            logo_url: None,
            description: "Meaningful test description".to_string(),
            tags: vec![Tag("Sci-Fi".to_string())],
            release_year: Some(2026),
        },
    }
}

// --- Fixed Tests ---

#[tokio::test]
async fn test_create_and_get_all_media_variants() -> std::result::Result<(), Error> {
    let db = setup_clean_db().await?;

    // Give every variant a distinct provider_id (101, 102, etc.)
    let variants = vec![
        (
            Media::Anime {
                studio: "Trigger".to_string(),
                episodes: 12,
            },
            "Cyberpunk",
            101,
        ),
        (
            Media::Movie {
                director: "Nolan".to_string(),
                duration: Duration::seconds(9000),
            },
            "Interstellar",
            102,
        ),
    ];

    for (media_variant, title, provider_id) in variants {
        let search_result = create_mock_search_result(media_variant, title, provider_id);

        let created = db.add(search_result).await?;
        assert!(created.id > 0);

        let fetched = db.get(created.id).await?;
        assert_eq!(fetched.metadata.title, title);
    }

    Ok(())
}

#[tokio::test]
async fn test_update_differential_logic() -> std::result::Result<(), Error> {
    let db = setup_clean_db().await?;
    let media = Media::Anime {
        studio: "Mappa".to_string(),
        episodes: 24,
    };
    let entry = db
        .add(create_mock_search_result(media, "Chainsaw Man", 201))
        .await?;

    // 1. Fresh baseline update
    db.update(
        entry.id,
        UpdateEntry {
            status: Some(Status::InProgress),
            rating: Some(Some(6)),
            notes: Some(Some("Masterpiece".to_string())),
            playtime: None,
        },
    )
    .await?;

    // 2. Partial update (None preserves existing values)
    let updated = db
        .update(
            entry.id,
            UpdateEntry {
                status: Some(Status::OnHold),
                rating: None,
                notes: None,
                playtime: None,
            },
        )
        .await?;

    assert!(matches!(updated.status, Status::OnHold));
    assert_eq!(updated.rating, Some(6));
    assert_eq!(updated.notes.as_deref(), Some("Masterpiece"));

    // 3. Explicit Nullification (Some(None) clears values)
    let wiped = db
        .update(
            entry.id,
            UpdateEntry {
                status: None,
                rating: Some(None),
                notes: Some(None),
                playtime: None,
            },
        )
        .await?;

    assert_eq!(wiped.rating, None);
    assert_eq!(wiped.notes, None);

    Ok(())
}

#[tokio::test]
async fn test_delete_purges_entry_meaningfully() -> std::result::Result<(), Error> {
    let db = setup_clean_db().await?;
    let media = Media::Movie {
        director: "Anno".to_string(),
        duration: Duration::seconds(6000),
    };
    // Clean, isolated ID for this test session
    let entry = db
        .add(create_mock_search_result(media, "Evangelion", 301))
        .await?;

    // Verify it works
    assert!(db.get(entry.id).await.is_ok());

    // Perform DELETE
    db.delete(entry.id).await?;

    // Confirm subsequent read fails strictly with Error::NotFound
    let post_delete_fetch = db.get(entry.id).await;
    assert!(
        matches!(post_delete_fetch, Err(Error::NotFound(id)) if id == entry.id),
        "Expected Error::NotFound after deletion, but got: {post_delete_fetch:?}"
    );

    Ok(())
}

#[tokio::test]
async fn test_get_nonexistent_returns_explicit_not_found() -> std::result::Result<(), Error> {
    let db = setup_clean_db().await?;
    let target_id = 404404;

    let result = db.get(target_id).await;

    assert!(
        matches!(result, Err(Error::NotFound(id)) if id == target_id),
        "Expected Err(Error::NotFound({target_id})), but got: {result:?}"
    );

    Ok(())
}

#[tokio::test]
async fn test_update_nonexistent_returns_explicit_not_found() -> std::result::Result<(), Error> {
    let db = setup_clean_db().await?;
    let target_id = 999999;

    let bad_update = UpdateEntry {
        status: Some(Status::Dropped),
        ..Default::default()
    };

    let result = db.update(target_id, bad_update).await;

    assert!(
        matches!(result, Err(Error::NotFound(id)) if id == target_id),
        "Expected Err(Error::NotFound({target_id})), but got: {result:?}"
    );

    Ok(())
}
