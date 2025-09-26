use std::collections::HashMap;
use std::sync::Arc;
use moka::future::Cache;
use crate::domain::entities::User;
use crate::domain::repositories::UserRepository;

fn id_key(issuer: &str, subject: &str) -> String {
    format!("{}::{}", issuer, subject)
}

pub struct UserResolver {
    user_repository: Arc<dyn UserRepository>,
    // cache by composite key now
    pub cache: Cache<String, Arc<User>>,
}

impl UserResolver {
    pub fn new(user_repository: Arc<dyn UserRepository>, cache: Cache<String, Arc<User>>) -> Self {
        Self { user_repository, cache }
    }

    pub async fn find_and_cache_user_by_identity(
        &self,
        issuer: &str,
        subject: &str,
    ) -> Result<Option<Arc<User>>, sqlx::Error> {
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

    pub async fn upsert_and_cache_user(
        &self,
        issuer: &str,
        subject: &str,
        username: &str,
        email: &str
    ) -> Result<Arc<User>, sqlx::Error> {
        let user = self.user_repository.upsert_user(issuer, subject, username, email).await?;
        let key = id_key(&user.idp_issuer, &user.idp_subject);
        let arc_user = Arc::new(user);
        self.cache.insert(key, arc_user.clone()).await;
        Ok(arc_user)
    }

    pub async fn remove_user_from_cache_and_db(&self, issuer: &str, subject: &str) -> Result<(), sqlx::Error> {
        let key = id_key(issuer, subject);
        self.cache.invalidate(&key).await;
        self.user_repository.delete_by_subject(issuer, subject).await
    }

    pub async fn get_all_users_mapped_to_key(&self) -> Result<HashMap<String, Arc<User>>, sqlx::Error> {
        let users = self.user_repository.get_all_users().await?;
        Ok(users.into_iter()
            .map(|u| (id_key(&u.idp_issuer, &u.idp_subject), Arc::new(u)))
            .collect())
    }

    pub async fn refresh_cache(&self) -> Result<(), anyhow::Error> {
        tracing::info!("Refreshing user cache from database");

        // Clear existing cache
        self.cache.invalidate_all();

        // Get all users from database and populate cache
        let users_map = self.get_all_users_mapped_to_key().await
            .map_err(|e| anyhow::anyhow!("Failed to refresh cache: {}", e))?;

        // Populate cache with all users
        for (key, user) in users_map {
            self.cache.insert(key, user).await;
        }

        tracing::info!("Cache refreshed with {} users", self.cache.entry_count());
        Ok(())
    }

    pub async fn get_all_identities(&self) -> Result<Vec<(String, String)>, anyhow::Error> {
        // Extract identities from all users
        let users = self.user_repository.get_all_users().await
            .map_err(|e| anyhow::anyhow!("Failed to fetch users: {}", e))?;

        Ok(users.into_iter()
            .map(|u| (u.idp_issuer, u.idp_subject))
            .collect())
    }
}
