use std::sync::Arc;
use std::time::Duration;
use moka::future::Cache;
use sqlx::PgPool;
use axum::extract::FromRef;

use crate::config::Config;
use crate::models::user::UserEntity;
use crate::repositories::user_repository::{PgUserRepo, UserRepository};
use crate::services::auth_service::AuthService;
use crate::services::user_service::UserService;
use crate::cache::user_resolver::UserResolver;
use crate::oidc::provider::OidcProvider;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub config: Config,
    pub db: Arc<PgPool>,
    pub auth_service: Arc<AuthService>,
    pub user_service: Arc<UserService>,
}

impl AppState {
    pub fn new(cfg: Config, pool: PgPool, provider: Arc<dyn OidcProvider + Send + Sync>) -> Self {
        let config = cfg.clone();
        let db = Arc::new(pool);

        let cache: Cache<String, Arc<UserEntity>> = Cache::builder()
            .time_to_live(Duration::from_secs(600))
            .max_capacity(10_000)
            .build();

        let user_repository: Arc<dyn UserRepository> = Arc::new(PgUserRepo::new(db.clone()));
        let user_resolver = Arc::new(UserResolver::new(user_repository.clone(), cache));

        let user_service = Arc::new(UserService::new(user_resolver.clone(), None));
        let auth_service = Arc::new(AuthService::new(provider, user_service.clone()));

        AppState { config, db, auth_service, user_service }
    }
}
