mod token;

use std::time::Instant;

use futures::lock::Mutex;
use time::UtcDateTime;

use super::*;
use crate::igdb::token::Token;

pub struct IGDBProvider {
    client: Client,
    token: Mutex<Option<Token>>,

    client_id: String,
    client_secret: String,
}

impl IGDBProvider {
    pub fn new(client_id: &str, client_secret: &str) -> Self {
        Self {
            client: Client::new(),
            token: Mutex::default(),

            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
        }
    }

    async fn access_token(&self) -> Result<String> {
        let mut token = self.token.lock().await;
        let needs_refresh = token.as_ref().is_none_or(|t| t.expires_in < Instant::now());

        if needs_refresh {
            let params = [
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("grant_type", "client_credentials"),
            ];

            let response: Token = self
                .client
                .post("https://id.twitch.tv/oauth2/token")
                .query(&params)
                .send()
                .await?
                .json::<TokenResponse>()
                .await?
                .into();

            *token = Some(response);
        }

        Ok(token.as_ref().unwrap().access_token.clone())
    }
}

impl ApiProvider for IGDBProvider {
    fn name(&self) -> &'static str {
        "IGDB"
    }

    fn kind(&self) -> MediaKind {
        MediaKind::Game
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let token = self.access_token().await?;

        let body = format!(
            r#"
            search "{}";
            fields 
                id,
                name,
                summary,
                storyline,
                genres.name,
                first_release_date,
                cover.image_id,
                artworks.image_id,
                involved_companies.company.name,
                game_type.type,
                involved_companies.developer;
            where game_type = 0 &
                genres != null &
                involved_companies != null;
            limit 5;
            "#,
            &query.query
        );

        let response = self
            .client
            .post("https://api.igdb.com/v4/games")
            .header("Client-ID", &self.client_id)
            .bearer_auth(token)
            .header("Content-Type", "text/plain")
            .body(body)
            .send()
            .await?
            .json::<Vec<RawGame>>()
            .await?;

        Ok(response
            .into_iter()
            .map(|game| {
                let media = ArtremisMedia::Game {
                    developer: game
                        .involved_companies
                        .into_iter()
                        .find_map(|x| x.developer.then_some(x.company.name)),

                    playtime: None,
                };

                let metadata = ProviderMetadata {
                    provider: self.name().to_string(),
                    provider_id: game.id,
                    title: game.name,

                    cover_url: game.cover.map(|x| image_url(&x.image_id, "cover_big")),
                    wide_url: game.artworks.and_then(|x| {
                        x.into_iter()
                            .next()
                            .map(|x| image_url(&x.image_id, "1080p"))
                    }),

                    description: game.storyline.or(game.summary),
                    tags: game.genres.into_iter().map(|x| x.name).collect(),

                    release_year: game.first_release_date.and_then(|x| {
                        UtcDateTime::from_unix_timestamp(x)
                            .map(|x| x.year().cast_unsigned())
                            .ok()
                    }),
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

#[derive(Deserialize, Debug)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

#[derive(Debug, Deserialize)]
struct RawGame {
    id: i64,
    name: String,
    summary: Option<String>,
    storyline: Option<String>,
    genres: Vec<Named>,
    first_release_date: Option<i64>,
    cover: Option<Image>,
    artworks: Option<Vec<Image>>,
    involved_companies: Vec<InvolvedCompany>,
}

#[derive(Debug, Deserialize)]
struct Named {
    name: String,
}

#[derive(Debug, Deserialize)]
struct Image {
    image_id: String,
}

#[derive(Debug, Deserialize)]
struct InvolvedCompany {
    company: Named,
    developer: bool,
}

fn image_url(hash: &str, size: &str) -> String {
    format!("https://images.igdb.com/igdb/image/upload/t_{size}/{hash}.jpg")
}
