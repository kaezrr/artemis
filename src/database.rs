use std::str::FromStr;

use sqlx::Sqlite;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::sqlite::SqlitePoolOptions;
use strum::IntoDiscriminant;

use crate::Error;
use crate::Result;
use crate::media::Collection;
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
use crate::query::CollectionAction;
use crate::query::LibraryQuery;
use crate::query::SortBy;
use crate::query::SortOrder;
use crate::query::TagFilter;
use crate::query::UpdateCollection;
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

        let media_result = sqlx::query(
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
        )
        .bind(search_result.media.discriminant())
        .bind(search_result.metadata.provider)
        .bind(search_result.metadata.provider_id)
        .bind(search_result.metadata.title)
        .bind(search_result.metadata.cover_url)
        .bind(search_result.metadata.wide_url)
        .bind(search_result.metadata.logo_url)
        .bind(search_result.metadata.description)
        .bind(search_result.metadata.release_year)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        let media_id = media_result.last_insert_rowid();

        match &search_result.media {
            Media::Anime { episodes, studio } => {
                sqlx::query("INSERT INTO anime_meta (media_id, studio, episodes) VALUES (?, ?, ?)")
                    .bind(media_id)
                    .bind(studio)
                    .bind(episodes)
            }

            Media::Movie { director, duration } => sqlx::query(
                "INSERT INTO movie_meta (media_id, director, duration) VALUES (?, ?, ?)",
            )
            .bind(media_id)
            .bind(director)
            .bind(duration),

            Media::Game {
                developer,
                playtime,
            } => sqlx::query(
                "INSERT INTO game_meta (media_id, developer, playtime) VALUES (?, ?, ?)",
            )
            .bind(media_id)
            .bind(developer)
            .bind(playtime),

            Media::TVShow { director, episodes } => sqlx::query(
                "INSERT INTO tvshow_meta (media_id, director, episodes) VALUES (?, ?, ?)",
            )
            .bind(media_id)
            .bind(director)
            .bind(episodes),
        }
        .execute(&mut *tx)
        .await?;

        for v in &search_result.metadata.tags {
            sqlx::query("INSERT INTO media_tag(media_id, tag) VALUES (?, ?)")
                .bind(media_id)
                .bind(v)
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
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(id),
            other => Error::DatabaseError(other),
        })?;

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

        let result = sqlx::query(
            "
            UPDATE media
            SET status = COALESCE(?2, status),
                notes  = CASE WHEN ?3 THEN ?4 ELSE notes END,
                rating = CASE WHEN ?5 THEN ?6 ELSE rating END
            WHERE id = ?1",
        )
        .bind(id)
        .bind(update.status)
        .bind(update.notes.is_some())
        .bind(update.notes.flatten())
        .bind(update.rating.is_some())
        .bind(update.rating.flatten())
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound(id));
        }

        if let Some(x) = &update.playtime {
            let result = sqlx::query("UPDATE game_meta SET playtime = ? WHERE media_id = ?")
                .bind(x)
                .bind(id)
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
        let result = sqlx::query("DELETE FROM media WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        (result.rows_affected() == 1)
            .then_some(())
            .ok_or(Error::NotFound(id))
    }

    pub async fn query(&self, query: LibraryQuery) -> Result<Vec<LibraryItem>> {
        let mut qb = sqlx::QueryBuilder::<Sqlite>::new(
            r#"
        SELECT 
            m.id,
            m.kind, m.title, m.cover_url,
            m.status, m.rating
        FROM media AS m
        WHERE 1 = 1"#,
        );

        if let Some(search) = &query.search {
            qb.push(" AND m.title LIKE ")
                .push_bind(format!("{search}%"));
        }

        if let Some(status) = &query.status {
            qb.push(" AND m.status = ").push_bind(status);
        }

        if let Some(kind) = &query.kind {
            qb.push(" AND m.kind = ").push_bind(kind);
        }

        if let Some(c_id) = &query.collection_id {
            qb.push(
                " AND EXISTS (
            SELECT 1
            FROM collection_media cm
            WHERE cm.media_id = m.id
              AND cm.collection_id = ",
            )
            .push_bind(c_id)
            .push(")");
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
                    qb.push(" AND m.id IN (SELECT media_id FROM media_tag WHERE tag IN (");

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
                SortBy::Title => "title",
                SortBy::Rating => "rating",
                SortBy::ReleaseYear => "release_year",
                SortBy::LastModified => "updated_at",
            })
            .push(match query.order {
                SortOrder::Ascending => " ASC",
                SortOrder::Descending => " DESC",
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

    pub async fn add_collection(&self, title: &str, media_ids: &[i64]) -> Result<Collection> {
        let mut tx = self.pool.begin().await?;

        let result = sqlx::query("INSERT INTO collection(title) VALUES(?)")
            .bind(title)
            .execute(&mut *tx)
            .await?;

        let collection_id = result.last_insert_rowid();

        for media_id in media_ids {
            sqlx::query(
                "INSERT OR IGNORE INTO collection_media(collection_id, media_id) VALUES (?, ?)",
            )
            .bind(collection_id)
            .bind(media_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(Collection {
            id: collection_id,
            title: title.to_string(),
            count: media_ids.len() as i64,
        })
    }

    pub async fn get_collection(&self, id: i64) -> Result<Collection> {
        sqlx::query_as(
            "SELECT 
                c.id,
                c.title,
                COUNT(cm.media_id) AS count
            FROM collection AS c
            LEFT JOIN collection_media AS cm
                ON c.id = cm.collection_id
            WHERE c.id = ?
            GROUP BY c.id",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(id),
            other => Error::DatabaseError(other),
        })
    }

    pub async fn get_collections(&self) -> Result<Vec<Collection>> {
        Ok(sqlx::query_as(
            "SELECT 
                c.id,
                c.title,
                COUNT(cm.media_id) AS count
            FROM collection AS c
            LEFT JOIN collection_media AS cm
                ON c.id = cm.collection_id
            GROUP BY c.id",
        )
        .fetch_all(&self.pool)
        .await?)
    }

    pub async fn update_collection(&self, id: i64, update: UpdateCollection) -> Result<Collection> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            "
            UPDATE collection
            SET title = COALESCE(?, title)
            WHERE id = ?",
        )
        .bind(update.title)
        .bind(id)
        .execute(&mut *tx)
        .await?;

        for entry_update in update.update_entries {
            match entry_update {
                CollectionAction::Add(media_id) => {
                    sqlx::query(
                        "INSERT OR IGNORE INTO collection_media (collection_id, media_id) VALUES (?, ?)",
                    )
                    .bind(id)
                    .bind(media_id)
                    .execute(&mut *tx)
                    .await?;
                }
                CollectionAction::Remove(media_id) => {
                    sqlx::query(
                        "DELETE FROM collection_media WHERE collection_id = ? AND media_id = ?",
                    )
                    .bind(id)
                    .bind(media_id)
                    .execute(&mut *tx)
                    .await?;
                }
            }
        }

        tx.commit().await?;

        self.get_collection(id).await
    }

    pub async fn delete_collection(&self, id: i64) -> Result<()> {
        let result = sqlx::query("DELETE FROM collection WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        (result.rows_affected() == 1)
            .then_some(())
            .ok_or(Error::NotFound(id))
    }
}
