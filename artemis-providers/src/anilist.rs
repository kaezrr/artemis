use super::*;

pub struct AnilistProvider {
    client: Client,
}

impl Default for AnilistProvider {
    fn default() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl ApiProvider for AnilistProvider {
    fn name(&self) -> &'static str {
        "AniList"
    }

    fn kind(&self) -> MediaKind {
        MediaKind::Anime
    }

    async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        let json = json!({
            "query": QUERY,
            "variables": {
                "search": query,
                "perPage": 5
            }
        });

        let response = self
            .client
            .post("https://graphql.anilist.co/")
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&json)
            .send()
            .await?
            .json::<Response>()
            .await?;

        Ok(response
            .data
            .page
            .media
            .into_iter()
            .map(|anime| {
                let media = ArtremisMedia::Anime {
                    studio: anime.studios.nodes.into_iter().next().map(|x| x.name),
                    episodes: anime.episodes,
                };

                let metadata = ProviderMetadata {
                    provider: self.name().to_string(),
                    provider_id: anime.id,
                    title: anime.title.english.unwrap_or(anime.title.romaji),
                    cover_url: anime.cover_image.extra_large,
                    wide_url: anime.banner_image,
                    description: anime.description,
                    tags: anime.genres,
                    release_year: anime.season_year,
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

const QUERY: &str = "
query ($search: String!, $perPage: Int) {
  Page(perPage: $perPage) {
    media(search: $search, type: ANIME, sort: POPULARITY_DESC) {
      id
      episodes
      genres
      coverImage {
        large
        extraLarge
      }
      studios(isMain: true) {
        nodes {
          name
        }
      }
      title {
        english
        romaji
      }
      seasonYear
      description(asHtml: false)
      bannerImage
    }
  }
}
";

#[derive(Debug, Deserialize)]
pub struct Response {
    pub data: Data,
}

#[derive(Debug, Deserialize)]
pub struct Data {
    #[serde(rename = "Page")]
    pub page: Page,
}

#[derive(Debug, Deserialize)]
pub struct Page {
    pub media: Vec<Media>,
}

#[derive(Debug, Deserialize)]
pub struct Media {
    pub id: i64,
    pub episodes: Option<u32>,
    pub genres: Vec<String>,
    #[serde(rename = "coverImage")]
    pub cover_image: CoverImage,
    pub studios: Studios,
    pub title: Title,
    #[serde(rename = "seasonYear")]
    pub season_year: Option<u32>,
    pub description: Option<String>,
    #[serde(rename = "bannerImage")]
    pub banner_image: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CoverImage {
    #[serde(rename = "extraLarge")]
    pub extra_large: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Studios {
    pub nodes: Vec<Studio>,
}

#[derive(Debug, Deserialize)]
pub struct Studio {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Title {
    pub english: Option<String>,
    pub romaji: String,
}
