use std::sync::Arc;
use sqlx::{Error};
use uuid::Uuid;
use crate::dto::auth::RegisterUserRequest;
use crate::user::resolver::UserResolver;
use crate::user::user::User;
use crate::user::user_repository;

#[derive(Clone)]
pub struct UserService {
    pub user_resolver: Arc<UserResolver>,
}

impl UserService {
    pub fn new(user_resolver: Arc<UserResolver> ) -> Self {
        Self { user_resolver }
    }

    pub async fn get_user_by_keycloak_id(&self, keycloak_id: Uuid) -> Result<Option<Arc<User>>, Error> {
        self.user_resolver.resolver_by_keycloak_id(keycloak_id).await
    }

    pub async fn create_user(
        &self,
        req: RegisterUserRequest,
        keycloak_id: Uuid
    ) -> Result<(), Error> {
        let new_user = User {
            id: Uuid::new_v4(),
            keycloak_id,
            username: req.username,
            email: req.email,
        };

        user_repository::insert_user(&self.user_resolver.db, &new_user).await
    }
}