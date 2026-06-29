mod app;
mod database;
mod media;
mod query;

pub use app::Application;
pub use media::LibraryEntry;
pub use media::SearchResult;
pub use query::Dashboard;
pub use query::LibraryQuery;
pub use query::SearchQuery;
pub use query::UpdateEntry;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("SQLite error: {0:?}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Error while migrating database: {0:?}")]
    MigrationError(#[from] sqlx::migrate::MigrateError),
    #[error("No entry with matching query was found")]
    NotFound,
}

pub type Result<T> = std::result::Result<T, Error>;
