use std::sync::Arc;
use moka::future::Cache;
use uuid::Uuid;
use crate::models::user::UserEntity;
use crate::repositories::user_repository::UserRepository;

pub struct UserResolver {
    user_repository: Arc<dyn UserRepository>,
    pub cache: Cache<Uuid, Arc<UserEntity>>,
}

impl UserResolver {
    pub fn new(user_repository: Arc<dyn UserRepository>, cache: Cache<Uuid, Arc<UserEntity>>) -> Self {
        Self { user_repository, cache }
    }

    pub async fn resolver_by_keycloak_id(
        &self,
        keycloak_id: Uuid,
    ) -> Result<Option<Arc<UserEntity>>, sqlx::Error> {
        if let Some(user) = self.cache.get(&keycloak_id).await {
            return Ok(Some(user));
        }

        let maybe_user = self.user_repository.find_by_keycloak_id(keycloak_id).await?;
        if let Some(u) = maybe_user {
            let arc_user = Arc::new(u);
            self.cache.insert(keycloak_id, arc_user.clone()).await;
            return Ok(Some(arc_user));
        }

        Ok(None)
    }

    /// Insert via repo, then cache
    pub async fn insert_and_cache_user(
        &self,
        new_user: UserEntity,
    ) -> Result<(), sqlx::Error> {
        // insert into DB
        self.user_repository.insert(&new_user).await?;

        // then cache it
        let arc_user = Arc::new(new_user);
        self.cache.insert(arc_user.keycloak_id, arc_user).await;
        Ok(())
    }

    pub async fn remove_user_from_cache_and_db(&self, keycloak_id: Uuid) -> Result<(), sqlx::Error> {
        self.cache.invalidate(&keycloak_id).await;
        self.user_repository.delete_by_keycloak_id(keycloak_id).await
    }

    pub async fn get_all_user_ids(&self) -> Result<Vec<Uuid>, sqlx::Error> {
        self.user_repository.get_all_user_ids().await
    }
}