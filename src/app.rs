use std::path::Path;

use crate::Result;
use crate::database::Database;
use crate::media::LibraryEntry;
use crate::media::SearchResult;
use crate::query::Dashboard;
use crate::query::LibraryQuery;
use crate::query::SearchQuery;
use crate::query::UpdateEntry;

pub struct Application {
    database: Database,
    // providers: Vec<dyn MediaProviders>
}

#[expect(unused)]
impl Application {
    pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            database: Database::open(path).await?,
        })
    }

    pub async fn add(&self, search_result: SearchResult) -> Result<LibraryEntry> {
        self.database.add(search_result).await
    }

    pub async fn get(&self, id: i64) -> Result<LibraryEntry> {
        self.database.get(id).await
    }

    pub async fn update(&self, id: i64, update: UpdateEntry) -> Result<LibraryEntry> {
        self.database.update(id, update).await
    }

    pub async fn delete(&self, id: i64) -> Result<()> {
        self.database.delete(id).await
    }

    pub async fn query(&self, query: LibraryQuery) -> Result<Vec<LibraryEntry>> {
        self.database.query(query).await
    }

    pub async fn random(&self, query: LibraryQuery) -> Result<Option<LibraryEntry>> {
        todo!()
    }

    pub async fn refresh(&self, id: i64) -> Result<LibraryEntry> {
        todo!()
    }

    pub async fn dashboard(&self) -> Result<Dashboard> {
        todo!()
    }

    pub async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>> {
        todo!()
    }
}
