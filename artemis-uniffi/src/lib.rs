uniffi::setup_scaffolding!();

mod provider;
mod types;

use artemis::Error;
use artemis::media::Collection;
use artemis::media::LibraryEntry;
use artemis::media::LibraryItem;
use artemis::media::SearchResult;
use artemis::query::Dashboard;
use artemis::query::LibraryQuery;
use artemis::query::UpdateCollection;
use artemis::query::UpdateEntry;

pub type Result<T> = std::result::Result<T, Error>;

#[uniffi::remote(Error)]
#[uniffi(flat_error)]
pub enum Error {
    DatabaseError,
    MigrationError,
    NotFound,
    TimeStampConversionError,
    ProviderError,
}

#[cfg(target_os = "android")]
#[jni::jni_mangle("dev.kaezr.artemis.RustPlatformVerifier")]
pub fn init<'caller>(
    mut unowned_env: jni::EnvUnowned<'caller>,
    _class: jni::objects::JClass<'caller>,
    context: jni::objects::JObject<'caller>,
) {
    unowned_env
        .with_env(|env| rustls_platform_verifier::android::init_with_env(env, context))
        .resolve::<jni::errors::ThrowRuntimeExAndDefault>();
}

#[derive(uniffi::Object)]
struct Application {
    app: artemis::Application,
}

#[uniffi::export]
impl Application {
    #[uniffi::constructor]
    pub async fn open(db_path: &str) -> Result<Self> {
        Ok(Self {
            app: artemis::Application::open(db_path).await?,
        })
    }

    pub async fn add(&self, search_result: SearchResult) -> Result<LibraryEntry> {
        self.app.add(search_result).await
    }

    pub async fn get(&self, id: i64) -> Result<LibraryEntry> {
        self.app.get(id).await
    }

    pub async fn update(&self, id: i64, update: UpdateEntry) -> Result<LibraryEntry> {
        self.app.update(id, update).await
    }

    pub async fn delete(&self, id: i64) -> Result<()> {
        self.app.delete(id).await
    }

    pub async fn query(&self, query: LibraryQuery) -> Result<Vec<LibraryItem>> {
        self.app.query(query).await
    }

    pub async fn random(&self, query: LibraryQuery) -> Result<Option<LibraryItem>> {
        self.app.random(query).await
    }

    pub async fn add_collection(&self, title: &str, media_ids: &[i64]) -> Result<Collection> {
        self.app.add_collection(title, media_ids).await
    }

    pub async fn get_collection(&self, id: i64) -> Result<Collection> {
        self.app.get_collection(id).await
    }

    pub async fn get_collections(&self) -> Result<Vec<Collection>> {
        self.app.get_collections().await
    }

    pub async fn update_collection(&self, id: i64, update: UpdateCollection) -> Result<Collection> {
        self.app.update_collection(id, update).await
    }

    pub async fn delete_collection(&self, id: i64) -> Result<()> {
        self.app.delete_collection(id).await
    }

    pub async fn tags_list(&self) -> Result<Vec<String>> {
        self.app.tags_list().await
    }

    pub async fn dashboard(&self) -> Result<Dashboard> {
        self.app.dashboard().await
    }

    pub async fn mark_search_results(
        &self,
        search_results: Vec<SearchResult>,
    ) -> Result<Vec<SearchResult>> {
        self.app.mark_search_results(search_results).await
    }
}
