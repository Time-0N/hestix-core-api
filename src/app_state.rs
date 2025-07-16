use std::sync::Arc;
use std::time::Duration;
use moka::future::Cache;
use sqlx::PgPool;
use uuid::Uuid;
use crate::config::Config;
use crate::models::user::UserEntity;
use crate::repositories::user_repository::{PgUserRepo, UserRepository};
use crate::security::keycloak::client::KeycloakClient;
use crate::security::keycloak::config::KeycloakConfig;
use crate::services::auth_service::AuthService;
use crate::services::user_service::UserService;
use crate::services::keycloak_service::KeycloakService;
use crate::cache::resolver::UserResolver;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: Arc<PgPool>,
    pub auth_service: Arc<AuthService>,
    pub user_service: Arc<UserService>,
}

impl AppState {
    pub fn new(cfg: Config, pool: PgPool) -> Self {
        let config = cfg.clone();
        let db = Arc::new(pool);

        let cache: Cache<Uuid, Arc<UserEntity>> = Cache::builder()
            .time_to_live(Duration::from_secs(600))
            .max_capacity(10_000)
            .build();

        let kc_cfg = KeycloakConfig {
            base_url:      cfg.keycloak_url,
            realm:         cfg.keycloak_realm,
            client_id:     cfg.keycloak_client_id,
            client_secret: cfg.keycloak_client_secret,
        };

        let user_repository: Arc<dyn UserRepository> = Arc::new(PgUserRepo::new(db.clone()));

        let user_resolver = Arc::new(UserResolver::new(user_repository.clone(), cache));

        let user_service = Arc::new(UserService::new(user_resolver.clone()));

        // 5) Build the KeycloakService from config
        let keycloak_client = KeycloakClient::new(kc_cfg);
        let keycloak_service = Arc::new(KeycloakService::new(keycloak_client));

        // 6) Build the AuthService using Keycloak + UserService
        let auth_service = Arc::new(AuthService::new(keycloak_service, user_service.clone()));

        AppState {
            config,
            db,
            auth_service,
            user_service,
        }
    }
}
