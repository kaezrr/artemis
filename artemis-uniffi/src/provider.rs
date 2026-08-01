use artemis::ApiProvider;
use artemis::Result;
use artemis::media::MediaKind;
use artemis::media::SearchResult;

macro_rules! provider_wrapper {
    ($wrapper:ident, $inner:path) => {
        #[derive(uniffi::Object)]
        struct $wrapper {
            inner: $inner,
        }

        #[uniffi::export]
        impl $wrapper {
            fn name(&self) -> String {
                self.inner.name().to_string()
            }

            fn kind(&self) -> MediaKind {
                self.inner.kind()
            }

            async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
                self.inner.search(&query).await
            }
        }
    };
}

provider_wrapper!(AnilistProvider, artemis_providers::AnilistProvider);
provider_wrapper!(TMDBMovieProvider, artemis_providers::TMDBMovieProvider);
provider_wrapper!(TMDBShowProvider, artemis_providers::TMDBShowProvider);
provider_wrapper!(IGDBProvider, artemis_providers::IGDBProvider);

// constructors still need their own impl blocks since signatures differ
#[uniffi::export]
impl AnilistProvider {
    #[uniffi::constructor]
    fn new() -> Self {
        AnilistProvider {
            inner: artemis_providers::AnilistProvider::default(),
        }
    }
}

#[uniffi::export]
impl TMDBMovieProvider {
    #[uniffi::constructor]
    fn new(api_key: &str) -> Self {
        TMDBMovieProvider {
            inner: artemis_providers::TMDBMovieProvider::new(api_key),
        }
    }
}

#[uniffi::export]
impl TMDBShowProvider {
    #[uniffi::constructor]
    fn new(api_key: &str) -> Self {
        TMDBShowProvider {
            inner: artemis_providers::TMDBShowProvider::new(api_key),
        }
    }
}

#[uniffi::export]
impl IGDBProvider {
    #[uniffi::constructor]
    fn new(client_id: &str, client_secret: &str) -> Self {
        IGDBProvider {
            inner: artemis_providers::IGDBProvider::new(client_id, client_secret),
        }
    }
}
