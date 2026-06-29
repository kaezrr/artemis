use std::path::Path;
use std::str::FromStr;

use sqlx::sqlite::SqliteConnectOptions;
use sqlx::sqlite::SqlitePoolOptions;
use strum::IntoDiscriminant;
use time::UtcDateTime;

use crate::Result;
use crate::media::Collection;
use crate::media::LibraryEntry;
use crate::media::Media;
use crate::media::SearchResult;
use crate::media::Status;
use crate::query::LibraryQuery;
use crate::query::UpdateEntry;

pub struct Database {
    pool: sqlx::SqlitePool,
}

impl Database {
    pub async fn open(path: &str) -> Result<Self> {
        let opts = SqliteConnectOptions::from_str(path)?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .pragma("foreign_keys", "ON");

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(opts)
            .await?;

        sqlx::migrate!("data/migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    pub async fn add(&self, search_result: SearchResult) -> Result<LibraryEntry> {
        let mut tx = self.pool.begin().await?;
        let now = UtcDateTime::now();

        let media_result = sqlx::query!(
            "INSERT INTO media (
                kind,
                provider,
                provider_id,
                title,
                cover_url,
                wide_url,
                logo_url,
                description,
                release_year,
                created_at,
                updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            &search_result.media.discriminant(),
            &search_result.metadata.provider,
            &search_result.metadata.provider_id,
            &search_result.metadata.title,
            &search_result.metadata.cover_url,
            &search_result.metadata.wide_url,
            &search_result.metadata.logo_url,
            &search_result.metadata.description,
            &search_result.metadata.release_year,
            &now.unix_timestamp(),
            &now.unix_timestamp(),
        )
        .execute(&mut *tx)
        .await?;

        let media_id = media_result.last_insert_rowid();

        match &search_result.media {
            Media::Anime { episodes, studio } => sqlx::query!(
                "INSERT INTO anime_meta (media_id, studio, episodes) VALUES (?, ?, ?)",
                media_id,
                studio,
                episodes
            ),

            Media::Movie { director, duration } => sqlx::query!(
                "INSERT INTO movie_meta (media_id, director, duration) VALUES (?, ?, ?)",
                media_id,
                director,
                duration.whole_seconds()
            ),

            Media::Game {
                developer,
                playtime,
            } => sqlx::query!(
                "INSERT INTO game_meta (media_id, developer, playtime) VALUES (?, ?, ?)",
                media_id,
                developer,
                playtime.map(|p| p.whole_seconds())
            ),

            Media::TVShow { director, episodes } => sqlx::query!(
                "INSERT INTO tvshow_meta (media_id, director, episodes) VALUES (?, ?, ?)",
                media_id,
                director,
                episodes,
            ),
        }
        .execute(&mut *tx)
        .await?;

        for v in &search_result.metadata.tags {
            sqlx::query!(
                "INSERT INTO media_tag(media_id, tag) VALUES (?, ?)",
                media_id,
                v,
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(LibraryEntry {
            id: media_id,

            media: search_result.media,
            metadata: search_result.metadata,

            rating: None,
            notes: None,
            status: Status::default(),

            created_at: now,
            updated_at: now,
        })
    }

    pub async fn get(&self, id: i64) -> Result<LibraryEntry> {
        todo!()
    }

    pub async fn update(&self, id: i64, update: UpdateEntry) -> Result<LibraryEntry> {
        todo!()
    }

    pub async fn delete(&self, id: i64) -> Result<()> {
        todo!()
    }

    pub async fn query(&self, query: LibraryQuery) -> Result<Vec<LibraryEntry>> {
        todo!()
    }
}
