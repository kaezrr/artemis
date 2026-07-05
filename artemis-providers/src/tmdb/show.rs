use super::*;

pub struct TMDBShowProvider {
    client: Client,
}

impl Default for TMDBShowProvider {
    fn default() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl ApiProvider for TMDBShowProvider {
    fn name(&self) -> &'static str {
        "TMDB"
    }

    fn kind(&self) -> MediaKind {
        MediaKind::TVShow
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        todo!()
    }
}
