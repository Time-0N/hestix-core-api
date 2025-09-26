use std::sync::Arc;
use std::time::Duration;
use moka::future::Cache;
use sqlx::PgPool;
use axum::extract::FromRef;
use reqwest::Client;
use tokio::sync::Mutex;
use crate::infrastructure::config::Config;
use crate::domain::entities::User;
use crate::infrastructure::persistence::{PgUserRepo, UserRepository};
use crate::application::auth_service::AuthService;
use crate::application::user_service::UserService;
use crate::infrastructure::oidc::provider::OidcProvider;
// ZitadelManagementClient import removed - now using trait
use crate::infrastructure::oidc::provider::OidcAdminApi;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub config: Config,
    pub db: Arc<PgPool>,
    pub auth_service: Arc<AuthService>,
    pub user_service: Arc<UserService>,
    pub http_client: Client,
}

impl AppState {
    pub fn new(cfg: Config, pool: PgPool, provider: Arc<dyn OidcProvider + Send + Sync>, http_client: Client, management_client: Option<Arc<Mutex<dyn OidcAdminApi + Send + Sync>>>) -> Self {
        let config = cfg.clone();
        let db = Arc::new(pool);

        let cache: Cache<String, Arc<User>> = Cache::builder()
            .time_to_live(Duration::from_secs(600))
            .max_capacity(10_000)
            .build();
        
        let http_client = http_client;

        let user_repository: Arc<dyn UserRepository> = Arc::new(PgUserRepo::new(db.clone()));

        let user_service = Arc::new(UserService::new(user_repository, cache, management_client, cfg.issuer_url.clone()));
        let auth_service = Arc::new(AuthService::new(provider, user_service.clone()));

        AppState { config, db, auth_service, user_service, http_client }
    }
}
