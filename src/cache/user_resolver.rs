use std::collections::HashMap;
use std::sync::Arc;
use moka::future::Cache;
use crate::models::user::UserEntity;
use crate::repositories::user_repository::UserRepository;

fn id_key(issuer: &str, subject: &str) -> String {
    format!("{}::{}", issuer, subject)
}

pub struct UserResolver {
    user_repository: Arc<dyn UserRepository>,
    // cache by composite key now
    pub cache: Cache<String, Arc<UserEntity>>,
}

impl UserResolver {
    pub fn new(user_repository: Arc<dyn UserRepository>, cache: Cache<String, Arc<UserEntity>>) -> Self {
        Self { user_repository, cache }
    }

    pub async fn find_and_cache_user_by_identity(
        &self,
        issuer: &str,
        subject: &str,
    ) -> Result<Option<Arc<UserEntity>>, sqlx::Error> {
        let key = id_key(issuer, subject);
        if let Some(user) = self.cache.get(&key).await {
            return Ok(Some(user));
        }

        let maybe_user = self.user_repository.find_by_subject(issuer, subject).await?;
        if let Some(u) = maybe_user {
            let arc_user = Arc::new(u);
            self.cache.insert(key, arc_user.clone()).await;
            return Ok(Some(arc_user));
        }
        Ok(None)
    }

    pub async fn insert_and_cache_user(&self, new_user: UserEntity) -> Result<(), sqlx::Error> {
        self.user_repository.insert(&new_user).await?;
        let key = id_key(&new_user.idp_issuer, &new_user.idp_subject);
        self.cache.insert(key, Arc::new(new_user)).await;
        Ok(())
    }

    pub async fn remove_user_from_cache_and_db(&self, issuer: &str, subject: &str) -> Result<(), sqlx::Error> {
        let key = id_key(issuer, subject);
        self.cache.invalidate(&key).await;
        self.user_repository.delete_by_subject(issuer, subject).await
    }

    pub async fn get_all_users_mapped_to_key(&self) -> Result<HashMap<String, Arc<UserEntity>>, sqlx::Error> {
        let users = self.user_repository.get_all_users().await?;
        Ok(users.into_iter()
            .map(|u| (id_key(&u.idp_issuer, &u.idp_subject), Arc::new(u)))
            .collect())
    }

    pub async fn update_and_cache_user(&self, user: UserEntity) -> Result<(), sqlx::Error> {
        self.user_repository.update_user(&user).await?;
        let key = id_key(&user.idp_issuer, &user.idp_subject);
        self.cache.insert(key, Arc::new(user)).await;
        Ok(())
    }
}
