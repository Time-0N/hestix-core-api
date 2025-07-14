use std::sync::Arc;
use crate::services::{
    auth_service::AuthService,
    keycloak_service::KeycloakService,
};

use sqlx::PgPool;
use crate::user::cache::new_user_cache;
use crate::user::resolver::UserResolver;
use crate::user::user_service::UserService;

pub struct ServiceBundle {
    pub auth_service: Arc<AuthService>,
    pub user_service: Arc<UserService>,
}

pub fn init_services(db_pool: Arc<PgPool>, keycloak_service: Arc<KeycloakService>) -> ServiceBundle {

    let user_cache = new_user_cache();

    let user_resolver = Arc::new(UserResolver::new(db_pool.clone(), user_cache));

    let user_service = Arc::new(UserService::new(user_resolver.clone()));

    let auth_service = Arc::new(AuthService::new(keycloak_service.clone(), user_service.clone()));

    ServiceBundle {
        auth_service,
        user_service,
    }
}

