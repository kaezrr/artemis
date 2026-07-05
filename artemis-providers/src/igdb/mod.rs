mod token;

use std::time::Instant;

use futures::lock::Mutex;

use super::*;
use crate::igdb::token::Token;

pub struct IGDBProvider {
    client: Client,
    token: Mutex<Option<Token>>,

    client_id: String,
    client_secret: String,
}

impl IGDBProvider {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client: Client::new(),
            token: Mutex::default(),

            client_id,
            client_secret,
        }
    }

    pub async fn access_token(&self) -> Result<String> {
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

#[async_trait::async_trait]
impl ApiProvider for IGDBProvider {
    fn name(&self) -> &'static str {
        "IGDB"
    }

    fn kind(&self) -> MediaKind {
        MediaKind::Game
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        todo!()
    }
}

#[derive(Deserialize, Debug)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}
