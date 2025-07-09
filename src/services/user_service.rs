use std::sync::Arc;
use sqlx::PgPool;
use uuid::Uuid;
use crate::dto::auth::RegisterUserRequest;
use crate::models::user::User;
use crate::repositories::user_repository;
use crate::security::keycloak::KeycloakError;
use crate::services::keycloak_service::KeycloakService;

#[derive(Clone)]
pub struct UserService {
    pub db: Arc<PgPool>,
    pub keycloak_service: Arc<KeycloakService>,
}

impl UserService {
    pub fn new(db: Arc<PgPool>, keycloak_service: Arc<KeycloakService> ) -> Self {
        Self { db, keycloak_service }
    }

    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<User>, sqlx::Error> {
        user_repository::find_user_by_id(&self.db, user_id).await
    }

    pub async fn register_user(
        &self,
        req: RegisterUserRequest,
    ) -> Result<(), KeycloakError> {
        let token = std::env::var("KEYCLOAK_ADMIN_TOKEN")
            .expect("KEYCLOAK_ADMIN_TOKEN must be set");

        self.keycloak_service
            .register_user(req, &token)
            .await
    }
}