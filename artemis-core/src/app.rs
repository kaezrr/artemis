use std::collections::HashMap;

use crate::Result;
use crate::database::Database;
use crate::media::Collection;
use crate::media::LibraryEntry;
use crate::media::LibraryItem;
use crate::media::SearchResult;
use crate::query::Dashboard;
use crate::query::LibraryQuery;
use crate::query::SortBy;
use crate::query::UpdateCollection;
use crate::query::UpdateEntry;

pub struct Application {
    database: Database,
}

impl Application {
    pub async fn open(db_path: &str) -> Result<Self> {
        Ok(Self {
            database: Database::open(db_path).await?,
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

    pub async fn tags_list(&self) -> Result<Vec<String>> {
        self.database.tags_list().await
    }

    pub async fn dashboard(&self) -> Result<Dashboard> {
        let recent = self
            .database
            .query(LibraryQuery {
                sort_by: SortBy::LastModified,
                limit: Some(5),
                ..Default::default()
            })
            .await?;

        let all_items = self.database.query(LibraryQuery::default()).await?;
        let mut media_counts = HashMap::new();

        all_items.iter().for_each(|x| {
            *media_counts.entry(x.kind).or_insert(0) += 1;
        });

        Ok(Dashboard {
            recent,
            media_counts,
        })
    }

    pub async fn mark_search_results(
        &self,
        mut search_results: Vec<SearchResult>,
    ) -> Result<Vec<SearchResult>> {
        let pairs: Vec<(&str, i64)> = search_results
            .iter()
            .map(|x| (x.metadata.provider.as_str(), x.metadata.provider_id))
            .collect();

        let existing = self.database.existing_ids(&pairs).await?;

        for result in search_results.iter_mut() {
            result.in_library = existing.contains(&(
                result.metadata.provider.clone(),
                result.metadata.provider_id,
            ));
        }

        Ok(search_results)
    }
}
