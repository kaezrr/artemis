use crate::media::MediaKind;
use crate::media::SearchResult;

cfg_select! {
    feature = "full" => {
        mod app;
        mod database;

        pub use app::Application;
        pub use database::Database;
    }
    _ => {}
}

pub mod media;
pub mod query;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("SQLite error: {0:?}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Error while migrating database: {0:?}")]
    MigrationError(#[from] sqlx::migrate::MigrateError),

    #[error("No entry with matching id {0} was found")]
    NotFound(i64),

    #[error("Timestamp could not be converted: {0:?}")]
    TimeStampConversionError(#[from] time::error::ComponentRange),

    #[error("Api provider failed: {0:?}")]
    ProviderError(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait ApiProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn kind(&self) -> MediaKind;
    fn search(&self, query: &str) -> impl Future<Output = Result<Vec<SearchResult>>> + Send;
}
