use std::path::Path;
use std::str::FromStr;

use sqlx::sqlite::SqliteConnectOptions;
use sqlx::sqlite::SqlitePoolOptions;

use crate::Result;
use crate::media::LibraryEntry;
use crate::media::SearchResult;
use crate::query::LibraryQuery;
use crate::query::UpdateEntry;

pub struct Database {
    pool: sqlx::SqlitePool,
}

impl Database {
    pub async fn open(_: impl AsRef<Path>) -> Result<Self> {
        // TODO: in memory for testing purposes right now
        let opts = SqliteConnectOptions::from_str("sqlite::memory:")?
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .pragma("foreign_keys", "ON");

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(opts)
            .await?;

        Ok(Self { pool })
    }

    pub async fn add(&self, search_result: SearchResult) -> Result<LibraryEntry> {
        todo!()
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
