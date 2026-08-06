use std::collections::HashMap;
use std::time::Duration;

use futures::StreamExt;
use futures::stream::FuturesUnordered;
use reqwest::Url;

use super::*;

pub struct TMDBMovieProvider {
    client: Client,
    api_key: String,
    base_url: Url,
}

impl TMDBMovieProvider {
    pub fn new(api_key: &str) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.to_string(),
            base_url: Url::parse("https://api.themoviedb.org/3/").unwrap(),
        }
    }
}

impl TMDBMovieProvider {
    fn name(&self) -> &'static str {
        "TMDB"
    }

    fn kind(&self) -> MediaKind {
        MediaKind::Movie
    }

    async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        let mut url = self.base_url.join("search/movie").unwrap();

        url.query_pairs_mut()
            .append_pair("query", query)
            .append_pair("include_adult", "true")
            .append_pair("language", "en-US")
            .append_pair("page", "1")
            .finish();

        let response = self
            .client
            .get(url)
            .bearer_auth(&self.api_key)
            .header("accept", "application/json")
            .send()
            .await?
            .json::<Response<Movie>>()
            .await?;

        let results: Vec<_> = response.results.into_iter().take(5).collect();
        let mut details = self.fetch_movie_details(&results).await?;

        Ok(results
            .into_iter()
            .map(|movie| {
                let details = details.remove(&movie.id).unwrap();

                let media = ArtremisMedia::Movie {
                    director: details.director,
                    duration: details.duration,
                };

                let metadata = ProviderMetadata {
                    provider: self.name().to_string(),
                    provider_id: movie.id,
                    title: movie.title,
                    cover_url: movie.poster_path.map(|x| tmdb_image_url(&x, "w500")),
                    wide_url: movie.backdrop_path.map(|x| tmdb_image_url(&x, "w1280")),
                    description: movie.overview,
                    tags: details.tags,
                    release_year: movie.release_date.get(..4).and_then(|x| x.parse().ok()),
                };

                SearchResult {
                    media,
                    metadata,
                    in_library: false,
                }
            })
            .collect())
    }
}

impl TMDBMovieProvider {
    async fn fetch_movie_details(&self, movies: &[Movie]) -> Result<HashMap<i64, MovieDetails>> {
        let mut hashmap = HashMap::with_capacity(movies.len());

        let mut futures =
            FuturesUnordered::from_iter(movies.iter().map(async move |movie| -> Result<_> {
                let mut url = self.base_url.join(&format!("movie/{}", movie.id)).unwrap();

                url.query_pairs_mut()
                    .append_pair("append_to_response", "credits")
                    .append_pair("language", "en-US")
                    .finish();

                let response = self
                    .client
                    .get(url)
                    .bearer_auth(&self.api_key)
                    .header("accept", "application/json")
                    .send()
                    .await?
                    .json::<MovieDetailsResponse>()
                    .await?;

                let director = response
                    .credits
                    .crew
                    .into_iter()
                    .find(|x| x.job == "Director")
                    .map(|x| x.name);

                let duration = response.runtime.map(u64::from).map(Duration::from_mins);
                let tags = response.genres.into_iter().map(|x| x.name).collect();

                Ok((
                    movie.id,
                    MovieDetails {
                        director,
                        duration,
                        tags,
                    },
                ))
            }));

        while let Some(result) = futures.next().await {
            let (id, details) = result?;
            hashmap.insert(id, details);
        }

        Ok(hashmap)
    }
}

struct MovieDetails {
    director: Option<String>,
    duration: Option<Duration>,
    tags: Vec<String>,
}

#[derive(Deserialize)]
struct MovieDetailsResponse {
    runtime: Option<u32>,
    genres: Vec<Genre>,
    credits: Credits,
}

#[derive(Debug, Deserialize)]
pub struct Movie {
    pub backdrop_path: Option<String>,
    pub id: i64,
    pub overview: Option<String>,
    pub poster_path: Option<String>,
    pub release_date: String,
    pub title: String,
}

#[derive(Deserialize)]
struct Credits {
    crew: Vec<CrewMember>,
}

#[derive(Deserialize)]
struct CrewMember {
    job: String,
    name: String,
}
