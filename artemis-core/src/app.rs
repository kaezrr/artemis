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
    /// Opens the library database at `db_path`, creating it if missing.
    ///
    /// # Errors
    ///
    /// Returns a database error if the connection or migrations fail.
    pub async fn open(db_path: &str) -> Result<Self> {
        Ok(Self {
            database: Database::open(db_path).await?,
        })
    }

    /// Adds a media entry to the library.
    ///
    /// # Errors
    ///
    /// Returns a database error if the insert fails.
    pub async fn add(&self, search_result: SearchResult) -> Result<LibraryEntry> {
        self.database.add(search_result).await
    }

    /// Returns the library entry with the given `id`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`](crate::Error::NotFound) if no entry matches `id`, otherwise a database error.
    pub async fn get(&self, id: i64) -> Result<LibraryEntry> {
        self.database.get(id).await
    }

    /// Updates the library entry with the given `id`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`](crate::Error::NotFound) if no entry matches `id`, otherwise a database error.
    pub async fn update(&self, id: i64, update: UpdateEntry) -> Result<LibraryEntry> {
        self.database.update(id, update).await
    }

    /// Removes the library entry with the given `id`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`](crate::Error::NotFound) if no entry matches `id`, otherwise a database error.
    pub async fn delete(&self, id: i64) -> Result<()> {
        self.database.delete(id).await
    }

    /// Queries the library for media entries.
    ///
    /// # Errors
    ///
    /// Returns a database error if the query fails.
    pub async fn query(&self, query: LibraryQuery) -> Result<Vec<LibraryItem>> {
        self.database.query(query).await
    }

    /// Returns a random entry matching `query`, or `None` if there are no matches.
    ///
    /// # Errors
    ///
    /// Returns a database error if the query fails.
    pub async fn random(&self, query: LibraryQuery) -> Result<Option<LibraryItem>> {
        let mut results = self.database.query(query).await?;

        if results.is_empty() {
            return Ok(None);
        }

        let random_index = fastrand::usize(..results.len());
        Ok(Some(results.swap_remove(random_index)))
    }

    /// Creates a new collection with the given `title` and `media_ids`.
    ///
    /// # Errors
    ///
    /// Returns a database error if the insert fails.
    pub async fn add_collection(&self, title: &str, media_ids: &[i64]) -> Result<Collection> {
        self.database.add_collection(title, media_ids).await
    }

    /// Returns the collection with the given `id`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`](crate::Error::NotFound) if no collection matches `id`, otherwise a database error.
    pub async fn get_collection(&self, id: i64) -> Result<Collection> {
        self.database.get_collection(id).await
    }

    /// Returns all collections.
    ///
    /// # Errors
    ///
    /// Returns a database error if the query fails.
    pub async fn get_collections(&self) -> Result<Vec<Collection>> {
        self.database.get_collections().await
    }

    /// Updates the collection with the given `id`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`](crate::Error::NotFound) if no collection matches `id`, otherwise a database error.
    pub async fn update_collection(&self, id: i64, update: UpdateCollection) -> Result<Collection> {
        self.database.update_collection(id, update).await
    }

    /// Removes the collection with the given `id`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`](crate::Error::NotFound) if no collection matches `id`, otherwise a database error.
    pub async fn delete_collection(&self, id: i64) -> Result<()> {
        self.database.delete_collection(id).await
    }

    /// Returns all tags used in the library.
    ///
    /// # Errors
    ///
    /// Returns a database error if the query fails.
    pub async fn tags_list(&self) -> Result<Vec<String>> {
        self.database.tags_list().await
    }

    /// Returns recent entries and per-kind counts for the dashboard.
    ///
    /// # Errors
    ///
    /// Returns a database error if either query fails.
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

        for x in all_items {
            *media_counts.entry(x.kind).or_insert(0) += 1;
        }

        Ok(Dashboard {
            recent,
            media_counts,
        })
    }

    /// Marks each result with whether it is already in the library.
    ///
    /// # Errors
    ///
    /// Returns a database error if the lookup fails.
    pub async fn mark_search_results(
        &self,
        mut search_results: Vec<SearchResult>,
    ) -> Result<Vec<SearchResult>> {
        let pairs: Vec<(&str, i64)> = search_results
            .iter()
            .map(|x| (x.metadata.provider.as_str(), x.metadata.provider_id))
            .collect();

        let existing = self.database.existing_ids(&pairs).await?;

        for result in &mut search_results {
            result.in_library = existing.contains(&(
                result.metadata.provider.clone(),
                result.metadata.provider_id,
            ));
        }

        Ok(search_results)
    }
}
