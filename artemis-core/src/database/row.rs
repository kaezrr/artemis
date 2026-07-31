use crate::media::MediaKind;
use crate::media::Status;
use crate::media::UtcDateTime;

#[derive(sqlx::FromRow)]
pub struct MediaRow {
    pub kind: MediaKind,
    pub provider: String,
    pub provider_id: i64,
    pub title: String,
    pub cover_url: Option<String>,
    pub wide_url: Option<String>,
    pub description: String,
    pub release_year: Option<u32>,
    pub rating: Option<u8>,
    pub notes: Option<String>,
    pub status: Status,
    pub created_at: UtcDateTime,
    pub updated_at: UtcDateTime,
}

#[derive(sqlx::FromRow)]
pub struct AnimeRow {
    pub studio: Option<String>,
    pub episodes: Option<u32>,
}

#[derive(sqlx::FromRow)]
pub struct MovieRow {
    pub director: Option<String>,
    pub duration: Option<i64>,
}

#[derive(sqlx::FromRow)]
pub struct GameRow {
    pub developer: Option<String>,
    pub playtime: Option<i64>,
}

#[derive(sqlx::FromRow)]
pub struct TVShowRow {
    pub creator: Option<String>,
    pub episodes: Option<u32>,
}
