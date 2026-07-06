use std::collections::HashMap;
use std::collections::HashSet;

use crate::Result;
use crate::database::Database;
use crate::media::Collection;
use crate::media::LibraryEntry;
use crate::media::LibraryItem;
use crate::media::SearchResult;
use crate::provider::ApiProvider;
use crate::provider::CombinedSearchProvider;
use crate::query::Dashboard;
use crate::query::LibraryQuery;
use crate::query::SearchQuery;
use crate::query::UpdateCollection;
use crate::query::UpdateEntry;

pub struct Application {
    database: Database,
    provider: CombinedSearchProvider,
}

impl Application {
    pub async fn open(path: &str) -> Result<Self> {
        Ok(Self {
            database: Database::open(path).await?,
            provider: CombinedSearchProvider::default(),
        })
    }

    pub fn add_provider(mut self, provider: Box<dyn ApiProvider>) -> Self {
        self.provider.add_provider(provider);
        self
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

    pub async fn random(&self, query: LibraryQuery) -> Result<Option<LibraryItem>> {
        let mut results = self.database.query(query).await?;

        if results.is_empty() {
            return Ok(None);
        }

        let random_index = fastrand::usize(..results.len());
        Ok(Some(results.swap_remove(random_index)))
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

    pub async fn dashboard(&self) -> Result<Dashboard> {
        todo!()
    }

    pub async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let mut results = self.provider.search(query).await?;

        let mut provider_to_ids = HashMap::<&str, Vec<i64>>::new();
        for r in &results {
            provider_to_ids
                .entry(&r.metadata.provider)
                .or_default()
                .push(r.metadata.provider_id);
        }

        let mut set = HashSet::<(String, i64)>::new();
        for (provider, ids) in provider_to_ids.into_iter() {
            let existing_ids = self.database.existing_ids(provider, &ids).await?;
            set.extend(existing_ids.into_iter().map(|id| (provider.to_owned(), id)));
        }

        for r in &mut results {
            r.in_library = set.contains(&(r.metadata.provider.clone(), r.metadata.provider_id));
        }

        Ok(results)
    }
}
