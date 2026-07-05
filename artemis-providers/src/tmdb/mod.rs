mod movie;
mod show;

pub use movie::TMDBMovieProvider;
pub use show::TMDBShowProvider;

use super::*;

fn tmdb_image_url(path: &str, size: &str) -> String {
    format!("http://image.tmdb.org/t/p/{size}/{path}")
}
