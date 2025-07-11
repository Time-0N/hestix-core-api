use std::sync::Arc;
use std::time::Duration;
use moka::future::Cache;
use sqlx::{Error, PgPool};
use uuid::Uuid;
use crate::dto::auth::RegisterUserRequest;
use crate::models::user::User;
use crate::repositories::user_repository;

#[derive(Clone)]
pub struct UserService {
    pub db: Arc<PgPool>,
    pub user_cache: Cache<Uuid, Arc<User>>,
}

impl UserService {
    pub fn new(db: Arc<PgPool>, ) -> Self {
        let user_cache = Cache::builder()
            .time_to_live(Duration::from_secs(60 * 10))
            .max_capacity(10_000)
            .build();

        Self {
            db,
            user_cache,
        }
    }

    pub async fn get_user_by_keycloak_id(&self, keycloak_id: Uuid) -> Result<Option<Arc<User>>, Error> {
        if let Some(user) = self.user_cache.get(&keycloak_id).await {
            return Ok(Some(user));
        }

        let user = user_repository::find_user_by_keycloak_id(&self.db, keycloak_id).await?;
        if let Some(ref u) = user {
            let user_arc = Arc::new(u.clone());
            self.user_cache.insert(keycloak_id, user_arc.clone()).await;
            return Ok(Some(user_arc));
        }

        Ok(None)
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

        user_repository::insert_user(&self.db, &new_user).await
    }
}