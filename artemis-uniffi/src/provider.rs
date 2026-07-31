#[derive(uniffi::Object)]
struct AnilistProvider {
    inner: artemis_providers::AnilistProvider,
}

#[uniffi::export]
impl AnilistProvider {
    #[uniffi::constructor]
    fn new() -> Self {
        AnilistProvider {
            inner: artemis_providers::AnilistProvider::default(),
        }
    }
}
