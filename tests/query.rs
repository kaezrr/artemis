use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use artemis::Database;
use artemis::Error;
use artemis::media::*;
use artemis::query::*;

static DB_COUNTER: AtomicUsize = AtomicUsize::new(0);

async fn setup_clean_db() -> Result<Database, Error> {
    let id = DB_COUNTER.fetch_add(1, Ordering::SeqCst);

    let db = Database::open(&format!("sqlite:file:memdb_{id}?mode=memory&cache=shared")).await?;

    Ok(db)
}

struct TestMedia {
    title: &'static str,
    media: Media,
    provider_id: i64,
    tags: Vec<&'static str>,
    status: Status,
    rating: Option<u8>,
}

fn titles(items: &[LibraryItem]) -> Vec<&str> {
    items.iter().map(|m| m.title.as_str()).collect()
}

fn create_search_result(test: &TestMedia) -> SearchResult {
    SearchResult {
        media: match &test.media {
            Media::Anime { studio, episodes } => Media::Anime {
                studio: studio.clone(),
                episodes: *episodes,
            },
            Media::Movie { director, duration } => Media::Movie {
                director: director.clone(),
                duration: *duration,
            },
            Media::Game {
                developer,
                playtime,
            } => Media::Game {
                developer: developer.clone(),
                playtime: *playtime,
            },
            Media::TVShow { director, episodes } => Media::TVShow {
                director: director.clone(),
                episodes: *episodes,
            },
        },

        metadata: ProviderMetadata {
            provider: "test".into(),
            provider_id: test.provider_id,

            title: test.title.into(),

            cover_url: format!("{}.jpg", test.title),
            wide_url: None,
            logo_url: None,

            description: format!("{} description", test.title),

            tags: test.tags.iter().map(|t| Tag((*t).to_string())).collect(),

            release_year: Some(2000 + test.provider_id as u32),
        },
    }
}

async fn populate_library(db: &Database) -> Result<Vec<LibraryEntry>, Error> {
    let library = vec![
        TestMedia {
            title: "Naruto",
            media: Media::Anime {
                studio: "Pierrot".into(),
                episodes: 220,
            },
            provider_id: 1,
            tags: vec!["Action", "Adventure"],
            status: Status::Finished,
            rating: Some(7),
        },
        TestMedia {
            title: "Naruto Shippuden",
            media: Media::Anime {
                studio: "Pierrot".into(),
                episodes: 500,
            },
            provider_id: 2,
            tags: vec!["Action"],
            status: Status::InProgress,
            rating: Some(6),
        },
        TestMedia {
            title: "Bleach",
            media: Media::Anime {
                studio: "Pierrot".into(),
                episodes: 366,
            },
            provider_id: 3,
            tags: vec!["Action", "Supernatural"],
            status: Status::Planned,
            rating: None,
        },
        TestMedia {
            title: "Interstellar",
            media: Media::Movie {
                director: "Christopher Nolan".into(),
                duration: Duration::seconds(10140),
            },
            provider_id: 4,
            tags: vec!["SciFi"],
            status: Status::Finished,
            rating: Some(7),
        },
        TestMedia {
            title: "Portal 2",
            media: Media::Game {
                developer: "Valve".into(),
                playtime: Some(Duration::seconds(20 * 3600)),
            },
            provider_id: 5,
            tags: vec!["Puzzle", "SciFi"],
            status: Status::Dropped,
            rating: Some(5),
        },
        TestMedia {
            title: "Breaking Bad",
            media: Media::TVShow {
                director: "Vince Gilligan".into(),
                episodes: 62,
            },
            provider_id: 6,
            tags: vec!["Crime"],
            status: Status::OnHold,
            rating: Some(7),
        },
    ];

    let mut entries = Vec::new();

    for media in library {
        let mut entry = db.add(create_search_result(&media)).await?;

        entry = db
            .update(
                entry.id,
                UpdateEntry {
                    status: Some(media.status),
                    rating: Some(media.rating),
                    ..Default::default()
                },
            )
            .await?;

        entries.push(entry);
    }

    Ok(entries)
}

#[tokio::test]
async fn query_empty_database_returns_empty() -> Result<(), Error> {
    let db = setup_clean_db().await?;

    let result = db.query(LibraryQuery::default()).await?;

    assert!(result.is_empty());

    Ok(())
}

#[tokio::test]
async fn query_default_returns_all_entries() -> Result<(), Error> {
    let db = setup_clean_db().await?;

    populate_library(&db).await?;

    let result = db.query(LibraryQuery::default()).await?;

    assert_eq!(result.len(), 6);

    Ok(())
}

#[tokio::test]
async fn prefix_search_returns_matching_titles() -> Result<(), Error> {
    let db = setup_clean_db().await?;

    populate_library(&db).await?;

    let result = db
        .query(LibraryQuery {
            search: Some("Nar".into()),
            ..Default::default()
        })
        .await?;

    assert_eq!(result.len(), 2);

    assert!(result.iter().any(|m| m.title == "Naruto"));
    assert!(result.iter().any(|m| m.title == "Naruto Shippuden"));

    Ok(())
}

#[tokio::test]
async fn prefix_search_is_case_insensitive() -> Result<(), Error> {
    let db = setup_clean_db().await?;

    populate_library(&db).await?;

    let result = db
        .query(LibraryQuery {
            search: Some("nAr".into()),
            ..Default::default()
        })
        .await?;

    assert_eq!(result.len(), 2);

    Ok(())
}

#[tokio::test]
async fn prefix_search_with_no_matches_returns_empty() -> Result<(), Error> {
    let db = setup_clean_db().await?;

    populate_library(&db).await?;

    let result = db
        .query(LibraryQuery {
            search: Some("One Piece".into()),
            ..Default::default()
        })
        .await?;

    assert!(result.is_empty());

    Ok(())
}

#[tokio::test]
async fn filter_by_anime_kind() -> Result<(), Error> {
    let db = setup_clean_db().await?;
    populate_library(&db).await?;

    let result = db
        .query(LibraryQuery {
            kind: Some(MediaKind::Anime),
            ..Default::default()
        })
        .await?;

    assert_eq!(
        titles(&result),
        vec!["Bleach", "Naruto", "Naruto Shippuden"]
    );

    Ok(())
}

#[tokio::test]
async fn filter_by_movie_kind() -> Result<(), Error> {
    let db = setup_clean_db().await?;
    populate_library(&db).await?;

    let result = db
        .query(LibraryQuery {
            kind: Some(MediaKind::Movie),
            ..Default::default()
        })
        .await?;

    assert_eq!(titles(&result), vec!["Interstellar"]);

    Ok(())
}

#[tokio::test]
async fn filter_by_status() -> Result<(), Error> {
    let db = setup_clean_db().await?;
    populate_library(&db).await?;

    let result = db
        .query(LibraryQuery {
            status: Some(Status::Finished),
            ..Default::default()
        })
        .await?;

    assert_eq!(titles(&result), vec!["Interstellar", "Naruto"]);

    Ok(())
}

#[tokio::test]
async fn tag_filter_or_returns_union() -> Result<(), Error> {
    let db = setup_clean_db().await?;
    populate_library(&db).await?;

    let result = db
        .query(LibraryQuery {
            tag_filter: Some(TagFilter::Or(vec![
                Tag("Crime".into()),
                Tag("SciFi".into()),
            ])),
            ..Default::default()
        })
        .await?;

    assert_eq!(
        titles(&result),
        vec!["Breaking Bad", "Interstellar", "Portal 2",]
    );

    Ok(())
}

#[tokio::test]
async fn tag_filter_and_returns_intersection() -> Result<(), Error> {
    let db = setup_clean_db().await?;
    populate_library(&db).await?;

    let result = db
        .query(LibraryQuery {
            tag_filter: Some(TagFilter::And(vec![
                Tag("Action".into()),
                Tag("Adventure".into()),
            ])),
            ..Default::default()
        })
        .await?;

    assert_eq!(titles(&result), vec!["Naruto"]);

    Ok(())
}

#[tokio::test]
async fn empty_or_tag_filter_does_nothing() -> Result<(), Error> {
    let db = setup_clean_db().await?;
    populate_library(&db).await?;

    let result = db
        .query(LibraryQuery {
            tag_filter: Some(TagFilter::Or(vec![])),
            ..Default::default()
        })
        .await?;

    assert_eq!(result.len(), 6);

    Ok(())
}

#[tokio::test]
async fn empty_and_tag_filter_does_nothing() -> Result<(), Error> {
    let db = setup_clean_db().await?;
    populate_library(&db).await?;

    let result = db
        .query(LibraryQuery {
            tag_filter: Some(TagFilter::And(vec![])),
            ..Default::default()
        })
        .await?;

    assert_eq!(result.len(), 6);

    Ok(())
}

#[tokio::test]
async fn combined_search_and_kind_filter() -> Result<(), Error> {
    let db = setup_clean_db().await?;
    populate_library(&db).await?;

    let result = db
        .query(LibraryQuery {
            search: Some("Nar".into()),
            kind: Some(MediaKind::Anime),
            ..Default::default()
        })
        .await?;

    assert_eq!(titles(&result), vec!["Naruto", "Naruto Shippuden"]);

    Ok(())
}

#[tokio::test]
async fn combined_kind_and_status_filter() -> Result<(), Error> {
    let db = setup_clean_db().await?;
    populate_library(&db).await?;

    let result = db
        .query(LibraryQuery {
            kind: Some(MediaKind::Anime),
            status: Some(Status::Finished),
            ..Default::default()
        })
        .await?;

    assert_eq!(titles(&result), vec!["Naruto"]);

    Ok(())
}

#[tokio::test]
async fn combined_search_status_and_tags() -> Result<(), Error> {
    let db = setup_clean_db().await?;
    populate_library(&db).await?;

    let result = db
        .query(LibraryQuery {
            search: Some("Nar".into()),
            status: Some(Status::Finished),
            tag_filter: Some(TagFilter::And(vec![Tag("Action".into())])),
            ..Default::default()
        })
        .await?;

    assert_eq!(titles(&result), vec!["Naruto"]);

    Ok(())
}

#[tokio::test]
async fn filters_with_no_matching_results_return_empty() -> Result<(), Error> {
    let db = setup_clean_db().await?;
    populate_library(&db).await?;

    let result = db
        .query(LibraryQuery {
            kind: Some(MediaKind::Movie),
            status: Some(Status::Dropped),
            ..Default::default()
        })
        .await?;

    assert!(result.is_empty());

    Ok(())
}

#[tokio::test]
async fn sort_by_title_descending() -> Result<(), Error> {
    let db = setup_clean_db().await?;
    populate_library(&db).await?;

    let result = db
        .query(LibraryQuery {
            sort_by: SortBy::Title,
            order: SortOrder::Descending,
            ..Default::default()
        })
        .await?;

    assert_eq!(
        titles(&result),
        vec![
            "Portal 2",
            "Naruto Shippuden",
            "Naruto",
            "Interstellar",
            "Breaking Bad",
            "Bleach",
        ]
    );

    Ok(())
}

#[tokio::test]
async fn sort_by_rating_ascending() -> Result<(), Error> {
    let db = setup_clean_db().await?;
    populate_library(&db).await?;

    let result = db
        .query(LibraryQuery {
            sort_by: SortBy::Rating,
            order: SortOrder::Ascending,
            ..Default::default()
        })
        .await?;

    assert_eq!(result.len(), 6);

    // SQLite sorts NULL first in ascending order.
    assert_eq!(result[0].title, "Bleach");
    assert_eq!(result[1].title, "Portal 2");
    assert_eq!(result[2].title, "Naruto Shippuden");

    Ok(())
}

#[tokio::test]
async fn sort_by_rating_descending() -> Result<(), Error> {
    let db = setup_clean_db().await?;
    populate_library(&db).await?;

    let result = db
        .query(LibraryQuery {
            sort_by: SortBy::Rating,
            order: SortOrder::Descending,
            ..Default::default()
        })
        .await?;

    assert_eq!(result.len(), 6);

    // SQLite sorts NULL last in descending order.
    assert_eq!(result.last().unwrap().title, "Bleach");

    Ok(())
}

#[tokio::test]
async fn sort_by_release_year_ascending() -> Result<(), Error> {
    let db = setup_clean_db().await?;
    populate_library(&db).await?;

    let result = db
        .query(LibraryQuery {
            sort_by: SortBy::ReleaseYear,
            order: SortOrder::Ascending,
            ..Default::default()
        })
        .await?;

    assert_eq!(
        titles(&result),
        vec![
            "Naruto",
            "Naruto Shippuden",
            "Bleach",
            "Interstellar",
            "Portal 2",
            "Breaking Bad",
        ]
    );

    Ok(())
}

#[tokio::test]
async fn sort_by_release_year_descending() -> Result<(), Error> {
    let db = setup_clean_db().await?;
    populate_library(&db).await?;

    let result = db
        .query(LibraryQuery {
            sort_by: SortBy::ReleaseYear,
            order: SortOrder::Descending,
            ..Default::default()
        })
        .await?;

    assert_eq!(
        titles(&result),
        vec![
            "Breaking Bad",
            "Portal 2",
            "Interstellar",
            "Bleach",
            "Naruto Shippuden",
            "Naruto",
        ]
    );

    Ok(())
}

#[tokio::test]
async fn sort_by_last_modified() -> Result<(), Error> {
    let db = setup_clean_db().await?;
    let entries = populate_library(&db).await?;

    // Touch Naruto so it becomes the newest modified entry.
    db.update(
        entries[0].id,
        UpdateEntry {
            notes: Some(Some("updated".into())),
            ..Default::default()
        },
    )
    .await?;

    let result = db
        .query(LibraryQuery {
            sort_by: SortBy::LastModified,
            order: SortOrder::Descending,
            ..Default::default()
        })
        .await?;

    assert_eq!(result.first().unwrap().title, "Naruto");

    Ok(())
}

#[tokio::test]
async fn limit_returns_requested_number_of_entries() -> Result<(), Error> {
    let db = setup_clean_db().await?;
    populate_library(&db).await?;

    let result = db
        .query(LibraryQuery {
            limit: Some(2),
            ..Default::default()
        })
        .await?;

    assert_eq!(result.len(), 2);

    Ok(())
}

#[tokio::test]
async fn offset_skips_entries() -> Result<(), Error> {
    let db = setup_clean_db().await?;
    populate_library(&db).await?;

    let all = db.query(LibraryQuery::default()).await?;

    let offset = db
        .query(LibraryQuery {
            offset: Some(2),
            ..Default::default()
        })
        .await?;

    assert_eq!(offset.len(), all.len() - 2);
    assert_eq!(titles(&offset), titles(&all)[2..]);

    Ok(())
}

#[tokio::test]
async fn limit_and_offset_work_together() -> Result<(), Error> {
    let db = setup_clean_db().await?;
    populate_library(&db).await?;

    let all = db.query(LibraryQuery::default()).await?;

    let result = db
        .query(LibraryQuery {
            limit: Some(2),
            offset: Some(2),
            ..Default::default()
        })
        .await?;

    assert_eq!(result.len(), 2);
    assert_eq!(titles(&result), titles(&all)[2..4]);

    Ok(())
}

#[tokio::test]
async fn offset_past_end_returns_empty() -> Result<(), Error> {
    let db = setup_clean_db().await?;
    populate_library(&db).await?;

    let result = db
        .query(LibraryQuery {
            offset: Some(100),
            ..Default::default()
        })
        .await?;

    assert!(result.is_empty());

    Ok(())
}

#[tokio::test]
async fn query_returns_expected_library_item_fields() -> Result<(), Error> {
    let db = setup_clean_db().await?;
    let entries = populate_library(&db).await?;

    let result = db.query(LibraryQuery::default()).await?;

    assert_eq!(result.len(), entries.len());

    for item in &result {
        assert!(item.id > 0);
        assert!(!item.title.is_empty());
        assert!(!item.cover_url.is_empty());

        match item.kind {
            MediaKind::Anime | MediaKind::Movie | MediaKind::Game | MediaKind::TVShow => {}
        }
    }

    Ok(())
}

#[tokio::test]
async fn returned_ids_match_inserted_entries() -> Result<(), Error> {
    let db = setup_clean_db().await?;
    let entries = populate_library(&db).await?;

    let expected: std::collections::HashSet<_> = entries.iter().map(|e| e.id).collect();

    let actual: std::collections::HashSet<_> = db
        .query(LibraryQuery::default())
        .await?
        .into_iter()
        .map(|e| e.id)
        .collect();

    assert_eq!(expected, actual);

    Ok(())
}
