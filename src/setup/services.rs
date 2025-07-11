use std::sync::Arc;
use crate::services::{
    auth_service::AuthService,
    user_service::UserService,
    keycloak_service::KeycloakService,
};

use sqlx::PgPool;

pub struct ServiceBundle {
    pub auth_service: Arc<AuthService>,
    pub user_service: Arc<UserService>,
}

pub fn init_services(db_pool: Arc<PgPool>, keycloak_service: Arc<KeycloakService>) -> ServiceBundle {
    let user_service = Arc::new(UserService::new(db_pool.clone()));
    let auth_service = Arc::new(AuthService::new(keycloak_service.clone(), user_service.clone()));

    ServiceBundle {
        auth_service,
        user_service,
    }
}

