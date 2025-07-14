use std::sync::Arc;
use moka::future::Cache;
use sqlx::PgPool;
use uuid::Uuid;
use crate::user::user::User;
use crate::user::user_repository;

pub struct UserResolver {
    pub db: Arc<PgPool>,
    pub cache: Cache<Uuid, Arc<User>>,
}

impl UserResolver {
    pub fn new (db: Arc<PgPool>, cache: Cache<Uuid, Arc<User>>) -> Self {
        Self { db, cache }
    }

    pub async fn resolver_by_keycloak_id(&self, keycloak_id: Uuid) -> Result<Option<Arc<User>>, sqlx::Error> {
        if let Some(user) = self.cache.get(&keycloak_id).await {
            return Ok(Some(user));
        }

        let user = user_repository::find_user_by_keycloak_id(&self.db, keycloak_id).await?;
        if let Some(u) = user {
            let arc_user = Arc::new(u);
            self.cache.insert(keycloak_id, arc_user.clone()).await;
            return Ok(Some(arc_user));
        }

        Ok(None)
    }
}