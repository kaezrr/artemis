use futures::StreamExt;
use futures::stream::FuturesUnordered;

use crate::Result;
use crate::media::MediaKind;
use crate::media::SearchResult;
use crate::query::SearchQuery;

#[async_trait::async_trait]
pub trait ApiProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn kind(&self) -> MediaKind;
    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>>;
}

#[derive(Default)]
pub struct CombinedSearchProvider {
    providers: Vec<Box<dyn ApiProvider>>,
}

impl CombinedSearchProvider {
    pub fn add_provider(&mut self, provider: Box<dyn ApiProvider>) {
        self.providers.push(provider);
    }

    pub fn remove_provider(&mut self, name: &str) {
        self.providers.retain(|x| x.name() != name);
    }

    pub async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let mut futures = self
            .providers
            .iter()
            .filter(|p| query.kind.is_none_or(|x| x == p.kind()))
            .map(|p| p.search(query))
            .collect::<FuturesUnordered<_>>();

        let mut results = Vec::new();

        while let Some(result) = futures.next().await {
            results.extend(result?);
        }

        Ok(results)
    }
}
