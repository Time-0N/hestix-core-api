// src/services/user_service.rs
use std::collections::HashMap;
use std::sync::Arc;
use sqlx::Error;
use tokio::sync::Mutex;
use moka::future::Cache;
use crate::domain::entities::User;
use crate::domain::repositories::UserRepository;
use crate::infrastructure::oidc::{OidcClaims, OidcError};
use crate::infrastructure::oidc::provider::OidcAdminApi;

fn id_key(issuer: &str, subject: &str) -> String {
    format!("{}::{}", issuer, subject)
}

#[derive(Clone)]
pub struct UserService {
    pub user_repository: Arc<dyn UserRepository>,
    pub cache: Cache<String, Arc<User>>,
    pub management_client: Option<Arc<Mutex<dyn OidcAdminApi + Send + Sync>>>,
    pub issuer_url: String,
}

impl UserService {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        cache: Cache<String, Arc<User>>,
        management_client: Option<Arc<Mutex<dyn OidcAdminApi + Send + Sync>>>,
        issuer_url: String,
    ) -> Self {
        Self {
            user_repository,
            cache,
            management_client,
            issuer_url,
        }
    }

    pub async fn get_user_by_identity(
        &self,
        issuer: &str,
        subject: &str,
    ) -> Result<Option<Arc<User>>, Error> {
        self.find_and_cache_user_by_identity(issuer, subject).await
    }

    pub async fn get_user_by_identity_bypass_cache(
        &self,
        issuer: &str,
        subject: &str,
    ) -> Result<Option<Arc<User>>, Error> {
        let maybe_user = self.user_repository.find_by_subject(issuer, subject).await?;
        if let Some(u) = maybe_user {
            let arc_user = Arc::new(u);
            // Update cache with fresh data
            let key = id_key(issuer, subject);
            self.cache.insert(key, arc_user.clone()).await;
            return Ok(Some(arc_user));
        }
        Ok(None)
    }

    async fn find_and_cache_user_by_identity(
        &self,
        issuer: &str,
        subject: &str,
    ) -> Result<Option<Arc<User>>, Error> {
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

    pub async fn sync_user_from_claims(
        &self,
        claims: &OidcClaims,
    ) -> Result<(), OidcError> {
        let issuer = &claims.iss;
        let sub = &claims.sub;

        let username = claims
            .preferred_username
            .clone()
            .or_else(|| claims.email.clone())
            .unwrap_or_else(|| sub.clone());

        let email = claims.email.clone()
            .ok_or_else(|| OidcError::Provider("Email is required".to_string()))?;

        self.upsert_and_cache_user(issuer, sub, &username, &email)
            .await
            .map_err(|e| OidcError::Provider(format!("User upsert failed: {}", e)))?;

        Ok(())
    }

    /// Full sync from ZITADEL Management API
    pub async fn sync_users(&self) -> anyhow::Result<()> {
        tracing::info!("Starting user sync");

        if let Some(client) = &self.management_client {
            tracing::info!("Fetching users from ZITADEL Management API");

            // Get all users from ZITADEL
            let client = client.lock().await;
            let users = client.fetch_all_users().await?;

            let mut synced_count = 0;
            let mut error_count = 0;

            // Sync each user to database
            for user in users {
                let username = user.username.unwrap_or_else(|| user.idp_subject.clone());
                let email = match user.email {
                    Some(email) => email,
                    None => {
                        tracing::warn!("Skipping user {} - no email provided", user.idp_subject);
                        continue;
                    }
                };

                match self.upsert_and_cache_user(&self.issuer_url, &user.idp_subject, &username, &email)
                    .await
                {
                    Ok(_) => synced_count += 1,
                    Err(e) => {
                        tracing::error!("Failed to sync user {}: {}", username, e);
                        error_count += 1;
                    }
                }
            }

            tracing::info!(
                "ZITADEL sync completed: {} users synced, {} errors", 
                synced_count, 
                error_count
            );

            // Optional: Remove users that no longer exist in ZITADEL
            // This is commented out for safety - enable if you want strict sync
            /*
            let zitadel_subjects: std::collections::HashSet<String> =
                users.into_iter().map(|u| u.idp_subject).collect();

            let db_identities = self.get_all_identities().await?;
            for (db_issuer, db_subject) in db_identities {
                if db_issuer == self.issuer_url && !zitadel_subjects.contains(&db_subject) {
                    tracing::warn!("Removing user {} - no longer in ZITADEL", db_subject);
                    self.remove_user_from_cache_and_db(&db_issuer, &db_subject).await?;
                }
            }
            */
        } else {
            // Just refresh cache from DB if no management client
            tracing::info!("No ZITADEL Management client - refreshing cache from database");
            self.refresh_cache().await?;
        }

        tracing::info!(
            "User cache contains {} users", 
            self.cache.entry_count()
        );

        Ok(())
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