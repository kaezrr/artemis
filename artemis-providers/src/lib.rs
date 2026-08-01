mod anilist;
mod igdb;
mod tmdb;

use artemis::ApiProvider;
use artemis::Result;
use artemis::media::Media as ArtremisMedia;
use artemis::media::MediaKind;
use artemis::media::ProviderMetadata;
use artemis::media::SearchResult;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

pub use crate::anilist::AnilistProvider;
pub use crate::igdb::IGDBProvider;
pub use crate::tmdb::TMDBMovieProvider;
pub use crate::tmdb::TMDBShowProvider;
