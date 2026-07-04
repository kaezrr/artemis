use crate::Result;
use crate::database::Database;
use crate::media::Collection;
use crate::media::LibraryEntry;
use crate::media::LibraryItem;
use crate::media::SearchResult;
use crate::query::Dashboard;
use crate::query::LibraryQuery;
use crate::query::SearchQuery;
use crate::query::UpdateCollection;
use crate::query::UpdateEntry;

pub struct Application {
    database: Database,
    // providers: Vec<dyn MediaProviders>
}

impl Application {
    pub async fn open(path: &str) -> Result<Self> {
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

    pub async fn query(&self, query: LibraryQuery) -> Result<Vec<LibraryItem>> {
        self.database.query(query).await
    }

    pub async fn random(&self, query: LibraryQuery) -> Result<LibraryItem> {
        let mut results = self.database.query(query).await?;
        let random_index = fastrand::usize(..results.len());
        Ok(results.swap_remove(random_index))
    }

    pub async fn add_collection(&self, title: &str, media_ids: &[i64]) -> Result<Collection> {
        self.database.add_collection(title, media_ids).await
    }
    pub async fn get_collection(&self, id: i64) -> Result<Collection> {
        self.database.get_collection(id).await
    }

    pub async fn get_collections(&self) -> Result<Vec<Collection>> {
        self.database.get_collections().await
    }
    pub async fn update_collection(&self, id: i64, update: UpdateCollection) -> Result<Collection> {
        self.database.update_collection(id, update).await
    }

    pub async fn delete_collection(&self, id: i64) -> Result<()> {
        self.database.delete_collection(id).await
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
