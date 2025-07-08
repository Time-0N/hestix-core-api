use crate::services::{
    auth_service::AuthService,
    user_service::UserService,
    keycloak_service::KeycloakService,
};

use sqlx::PgPool;

pub struct ServiceBundle {
    pub auth_service: AuthService,
    pub user_service: UserService,
    pub keycloak_service: KeycloakService,
}

pub fn init_services(db_pool: PgPool, keycloak_service: KeycloakService) -> ServiceBundle {
    let user_service = UserService::new(db_pool.clone(), keycloak_service.clone());
    let auth_service = AuthService::new(keycloak_service.clone(), user_service.clone());

    ServiceBundle {
        auth_service,
        user_service,
        keycloak_service,
    }
}

