uniffi::setup_scaffolding!();

mod types;

use artemis::Error;
use artemis::media::Collection;
use artemis::media::LibraryEntry;
use artemis::media::LibraryItem;
use artemis::media::SearchResult;
use artemis::query::Dashboard;
use artemis::query::LibraryQuery;
use artemis::query::UpdateCollection;
use artemis::query::UpdateEntry;

pub type Result<T> = std::result::Result<T, Error>;

/// Errors surfaced across the FFI boundary.
///
/// `NotFound` is returned when an entry or collection id doesn't exist;
/// the remaining variants cover database, migration, and timestamp failures.
#[uniffi::remote(Error)]
#[uniffi(flat_error)]
pub enum Error {
    DatabaseError,
    MigrationError,
    NotFound,
    TimeStampConversionError,
}

#[derive(uniffi::Object)]
struct Application {
    app: artemis::Application,
}

#[uniffi::export]
impl Application {
    /// Opens the library at `db_path`, creating the database if missing.
    ///
    /// Fails with a database or migration error if the file can't be opened.
    #[uniffi::constructor]
    pub async fn open(db_path: &str) -> Result<Self> {
        Ok(Self {
            app: artemis::Application::open(db_path).await?,
        })
    }

    /// Adds a media entry to the library and returns the stored entry.
    ///
    /// Duplicates of the same `provider` + `provider_id` are rejected.
    pub async fn add(&self, search_result: SearchResult) -> Result<LibraryEntry> {
        self.app.add(search_result).await
    }

    /// Returns the entry with the given `id`.
    ///
    /// Fails with `NotFound` if no entry matches.
    pub async fn get(&self, id: i64) -> Result<LibraryEntry> {
        self.app.get(id).await
    }

    /// Applies a partial [`UpdateEntry`] and returns the updated entry.
    ///
    /// Fails with `NotFound` if no entry matches.
    pub async fn update(&self, id: i64, update: UpdateEntry) -> Result<LibraryEntry> {
        self.app.update(id, update).await
    }

    /// Removes the entry with the given `id`.
    ///
    /// Fails with `NotFound` if no entry matches.
    pub async fn delete(&self, id: i64) -> Result<()> {
        self.app.delete(id).await
    }

    /// Queries the library, returning matching entries as [`LibraryItem`]s.
    pub async fn query(&self, query: LibraryQuery) -> Result<Vec<LibraryItem>> {
        self.app.query(query).await
    }

    /// Returns a random entry matching `query`, or `None` if nothing matches.
    pub async fn random(&self, query: LibraryQuery) -> Result<Option<LibraryItem>> {
        self.app.random(query).await
    }

    /// Creates a collection with the given `title`, optionally pre-filling
    /// it with the given entry ids.
    pub async fn add_collection(&self, title: &str, media_ids: &[i64]) -> Result<Collection> {
        self.app.add_collection(title, media_ids).await
    }

    /// Returns the collection with the given `id`.
    ///
    /// Fails with `NotFound` if no collection matches.
    pub async fn get_collection(&self, id: i64) -> Result<Collection> {
        self.app.get_collection(id).await
    }

    /// Returns all collections.
    pub async fn get_collections(&self) -> Result<Vec<Collection>> {
        self.app.get_collections().await
    }

    /// Applies a partial [`UpdateCollection`] and returns the updated
    /// collection.
    ///
    /// Fails with `NotFound` if no collection matches.
    pub async fn update_collection(&self, id: i64, update: UpdateCollection) -> Result<Collection> {
        self.app.update_collection(id, update).await
    }

    /// Removes the collection with the given `id`.
    ///
    /// Fails with `NotFound` if no collection matches.
    pub async fn delete_collection(&self, id: i64) -> Result<()> {
        self.app.delete_collection(id).await
    }

    /// Returns every tag used across the library, sorted.
    pub async fn tags_list(&self) -> Result<Vec<String>> {
        self.app.tags_list().await
    }

    /// Returns the dashboard snapshot: recent entries and per-kind counts.
    pub async fn dashboard(&self) -> Result<Dashboard> {
        self.app.dashboard().await
    }

    /// Fills in each result's `in_library` flag based on what's already
    /// saved, returning the annotated results.
    pub async fn mark_search_results(
        &self,
        search_results: Vec<SearchResult>,
    ) -> Result<Vec<SearchResult>> {
        self.app.mark_search_results(search_results).await
    }
}
