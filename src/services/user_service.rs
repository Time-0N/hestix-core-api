use std::sync::Arc;
use sqlx::{Error, PgPool};
use uuid::Uuid;
use crate::dto::auth::RegisterUserRequest;
use crate::models::user::User;
use crate::repositories::user_repository;
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

    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<User>, Error> {
        user_repository::find_user_by_id(&self.db, user_id).await
    }

    pub async fn create_user(
        &self,
        req: RegisterUserRequest,
        keycloak_id: String
    ) -> Result<(), Error> {
        let new_user = User {
            id: Uuid::new_v4(),
            keycloak_id,
            username: req.username,
            email: req.email,
        };

        user_repository::insert_user(&self.db, &new_user).await
    }
}