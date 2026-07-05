use artemis::provider::ApiProvider;
use reqwest::Client;

struct AnilistProvider {
    client: Client,
}
