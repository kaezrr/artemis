use std::collections::HashMap;

use futures::StreamExt;
use futures::stream::FuturesUnordered;
use reqwest::Url;

use super::*;

pub struct TMDBShowProvider {
    client: Client,
    api_key: String,
    base_url: Url,
}

impl TMDBShowProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: Url::parse("https://api.themoviedb.org/3/").unwrap(),
        }
    }
}

impl ApiProvider for TMDBShowProvider {
    fn name(&self) -> &'static str {
        "TMDB"
    }

    fn kind(&self) -> MediaKind {
        MediaKind::TVShow
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let mut url = self.base_url.join("search/tv").unwrap();

        url.query_pairs_mut()
            .append_pair("query", query.query.as_str())
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
            .json::<Response<TVShow>>()
            .await?;

        let results: Vec<_> = response.results.into_iter().take(5).collect();
        let mut details = self.fetch_show_details(&results).await?;

        Ok(results
            .into_iter()
            .map(|show| {
                let details = details.remove(&show.id).unwrap();

                let media = ArtremisMedia::TVShow {
                    creator: details.creator,
                    episodes: details.episodes,
                };

                let metadata = ProviderMetadata {
                    provider: self.name().to_string(),
                    provider_id: show.id,
                    title: show.name,
                    cover_url: show.poster_path.map(|x| tmdb_image_url(&x, "w500")),
                    wide_url: show.backdrop_path.map(|x| tmdb_image_url(&x, "w1280")),
                    description: show.overview,
                    tags: details.tags,
                    release_year: show.first_air_date.get(..4).and_then(|x| x.parse().ok()),
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

impl TMDBShowProvider {
    async fn fetch_show_details(&self, shows: &[TVShow]) -> Result<HashMap<i64, ShowDetails>> {
        let mut hashmap = HashMap::with_capacity(shows.len());

        let mut futures =
            FuturesUnordered::from_iter(shows.iter().map(async move |movie| -> Result<_> {
                let mut url = self.base_url.join(&format!("tv/{}", movie.id)).unwrap();

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
                    .json::<ShowDetailsResponse>()
                    .await?;

                let creator = response.created_by.into_iter().next().map(|x| x.name);
                let tags = response.genres.into_iter().map(|x| x.name).collect();

                Ok((
                    movie.id,
                    ShowDetails {
                        creator,
                        episodes: response.number_of_episodes,
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

struct ShowDetails {
    creator: Option<String>,
    episodes: Option<u32>,
    tags: Vec<String>,
}

#[derive(Deserialize)]
struct ShowDetailsResponse {
    number_of_episodes: Option<u32>,
    genres: Vec<Genre>,
    created_by: Vec<Person>,
}

#[derive(Deserialize)]
struct Person {
    name: String,
}

#[derive(Debug, Deserialize)]
pub struct TVShow {
    pub backdrop_path: Option<String>,
    pub id: i64,
    pub overview: Option<String>,
    pub poster_path: Option<String>,
    pub first_air_date: String,
    pub name: String,
}
