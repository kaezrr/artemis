use super::*;

pub struct IGDBProvider {
    client: Client,
}

impl Default for IGDBProvider {
    fn default() -> Self {
        Self {
            client: Client::new(),
        }
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
