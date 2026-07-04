mod app;
mod database;
pub mod media;
pub mod provider;
pub mod query;

pub use app::Application;
pub use database::Database;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("SQLite error: {0:?}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Error while migrating database: {0:?}")]
    MigrationError(#[from] sqlx::migrate::MigrateError),
    #[error("No entry with matching id {0} was found")]
    NotFound(i64),
    #[error("Api provider {name} failed: {source:?}")]
    ProviderError {
        name: String,
        source: reqwest::Error,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
