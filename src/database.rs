use std::str::FromStr;

use sqlx::Sqlite;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::sqlite::SqlitePoolOptions;
use strum::IntoDiscriminant;

use crate::Error;
use crate::Result;
use crate::media::Duration;
use crate::media::LibraryEntry;
use crate::media::LibraryItem;
use crate::media::Media;
use crate::media::MediaKind;
use crate::media::ProviderMetadata;
use crate::media::SearchResult;
use crate::media::Status;
use crate::media::Tag;
use crate::media::UtcDateTime;
use crate::query::LibraryQuery;
use crate::query::TagFilter;
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
            &now,
            &now,
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
                duration,
            ),

            Media::Game {
                developer,
                playtime,
            } => sqlx::query!(
                "INSERT INTO game_meta (media_id, developer, playtime) VALUES (?, ?, ?)",
                media_id,
                developer,
                playtime,
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

        self.get(media_id).await
    }

    pub async fn get(&self, id: i64) -> Result<LibraryEntry> {
        let entry = sqlx::query!(
            r#"SELECT
            kind as "kind: MediaKind",
            provider,
            provider_id,
            title,
            cover_url,
            wide_url,
            logo_url,
            description,
            release_year as "release_year: u32",
            rating as "rating: u8", 
            notes,
            status as "status: Status",
            created_at as "created_at: UtcDateTime",
            updated_at as "updated_at: UtcDateTime"
            FROM media WHERE id = ?"#,
            &id
        )
        .fetch_one(&self.pool)
        .await?;

        let tags: Vec<Tag> = sqlx::query_scalar("SELECT tag FROM media_tag WHERE media_id = $1")
            .bind(id)
            .fetch_all(&self.pool)
            .await?;

        let media = match entry.kind {
            MediaKind::Anime => {
                sqlx::query_as!(
                    Media::Anime,
                    r#"SELECT studio, episodes as "episodes: u32" FROM anime_meta WHERE media_id = ?"#,
                    id
                )
                .fetch_one(&self.pool)
                .await?
            }
            MediaKind::Movie => {
                sqlx::query_as!(
                    Media::Movie,
                    r#"SELECT director, duration as "duration: Duration" FROM movie_meta WHERE media_id = ?"#,
                    id
                )
                .fetch_one(&self.pool)
                .await?
            }
            MediaKind::Game => {
                sqlx::query_as!(
                    Media::Game,
                    r#"SELECT developer, playtime as "playtime: Duration" FROM game_meta WHERE media_id = ?"#,
                    id
                )
                .fetch_one(&self.pool)
                .await?
            }
            MediaKind::TVShow => {
                sqlx::query_as!(
                    Media::TVShow,
                    r#"SELECT director, episodes as "episodes: u32" FROM tvshow_meta WHERE media_id = ?"#,
                    id
                )
                .fetch_one(&self.pool)
                .await?
            }
        };

        Ok(LibraryEntry {
            id,
            media,

            metadata: ProviderMetadata {
                provider: entry.provider,
                provider_id: entry.provider_id,
                title: entry.title,
                cover_url: entry.cover_url,
                wide_url: entry.wide_url,
                logo_url: entry.logo_url,
                description: entry.description,
                tags,
                release_year: entry.release_year,
            },

            rating: entry.rating,
            notes: entry.notes,
            status: entry.status,

            created_at: entry.created_at,
            updated_at: entry.updated_at,
        })
    }

    pub async fn update(&self, id: i64, update: UpdateEntry) -> Result<LibraryEntry> {
        let mut tx = self.pool.begin().await?;

        let result = sqlx::query!(
            r#"
            UPDATE media
            SET status = COALESCE(?2, status),
                notes  = CASE WHEN ?3 THEN ?4 ELSE notes END,
                rating = CASE WHEN ?5 THEN ?6 ELSE rating END
            WHERE id = ?1"#,
            id,
            update.status,
            update.notes.is_some(),
            update.notes.flatten(),
            update.rating.is_some(),
            update.rating.flatten(),
        )
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound(id));
        }

        if let Some(x) = &update.playtime {
            let result = sqlx::query!(
                "UPDATE game_meta SET playtime = ? WHERE media_id = ?",
                &x,
                &id
            )
            .execute(&mut *tx)
            .await?;

            if result.rows_affected() == 0 {
                return Err(Error::NotFound(id));
            }
        }

        tx.commit().await?;

        self.get(id).await
    }

    pub async fn delete(&self, id: i64) -> Result<()> {
        let result = sqlx::query!("DELETE FROM media WHERE id = ?", &id)
            .execute(&self.pool)
            .await?;

        (result.rows_affected() == 0)
            .then_some(())
            .ok_or(Error::NotFound(id))
    }

    pub async fn query(&self, query: LibraryQuery) -> Result<Vec<LibraryItem>> {
        let mut qb = sqlx::QueryBuilder::<Sqlite>::new(
            r#"
        SELECT 
            id,
            kind, title, cover_url,
            status, rating
        FROM media
        WHERE 1 = 1"#,
        );

        if let Some(search) = &query.search {
            qb.push(" AND title LIKE ").push_bind(format!("{search}%"));
        }

        if let Some(status) = &query.status {
            qb.push(" AND status = ").push_bind(status);
        }

        if let Some(kind) = &query.kind {
            qb.push(" AND kind = ").push_bind(kind);
        }

        if let Some(tag_filter) = &query.tag_filter {
            match tag_filter {
                TagFilter::Or(tags) if !tags.is_empty() => {
                    qb.push(" AND id IN (SELECT media_id FROM media_tag WHERE tag IN (");

                    let mut separated = qb.separated(", ");
                    for tag in tags.iter() {
                        separated.push_bind(tag);
                    }
                    separated.push_unseparated(") ");

                    qb.push(") ");
                }

                TagFilter::And(tags) if !tags.is_empty() => {
                    qb.push(" AND id IN (SELECT media_id FROM media_tag WHERE tag IN (");

                    let mut separated = qb.separated(", ");
                    for tag in tags.iter() {
                        separated.push_bind(tag);
                    }
                    separated.push_unseparated(") ");

                    qb.push(" GROUP BY media_id");
                    qb.push(" HAVING COUNT(DISTINCT tag) = ")
                        .push_bind(tags.len() as i64);

                    qb.push(") ");
                }

                _ => {}
            }
        }

        qb.push(" ORDER BY ")
            .push(match query.sort_by {
                crate::query::SortBy::Title => "title",
                crate::query::SortBy::Rating => "rating",
                crate::query::SortBy::ReleaseYear => "release_year",
                crate::query::SortBy::LastModified => "updated_at",
            })
            .push(match query.order {
                crate::query::SortOrder::Ascending => " ASC",
                crate::query::SortOrder::Descending => " DESC",
            });

        if let Some(limit) = query.limit {
            qb.push(" LIMIT ").push_bind(limit);
        }

        if let Some(offset) = query.offset {
            if query.limit.is_none() {
                qb.push(" LIMIT -1 ");
            }
            qb.push(" OFFSET ").push_bind(offset);
        }

        Ok(qb.build_query_as().fetch_all(&self.pool).await?)
    }
}
