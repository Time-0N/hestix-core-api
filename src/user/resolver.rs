use std::sync::Arc;
use moka::future::Cache;
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::user::UserEntity;
use crate::repositories::user_repository;

pub struct UserResolver {
    pub db: Arc<PgPool>,
    pub cache: Cache<Uuid, Arc<UserEntity>>,
}

impl UserResolver {
    pub fn new (db: Arc<PgPool>, cache: Cache<Uuid, Arc<UserEntity>>) -> Self {
        Self { db, cache }
    }

    pub async fn resolver_by_keycloak_id(&self, keycloak_id: Uuid) -> Result<Option<Arc<UserEntity>>, sqlx::Error> {
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

    pub async fn insert_and_cache_user(
        &self,
        user: UserEntity,
    ) -> Result<(), sqlx::Error> {
        let arc_user = Arc::new(user);
        user_repository::insert_user(&self.db, &arc_user).await?;
        self.cache.insert(arc_user.keycloak_id, arc_user).await;
        Ok(())
    }
}