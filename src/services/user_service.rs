// src/services/user_service.rs
use std::sync::Arc;
use sqlx::Error;
use tokio::sync::Mutex;
use crate::util::cache::user_resolver::UserResolver;
use crate::model::user::UserEntity;
use crate::util::oidc::{OidcClaims, OidcError};
use crate::util::oidc::providers::zitadel::management::ZitadelManagementClient;

#[derive(Clone)]
pub struct UserService {
    pub user_resolver: Arc<UserResolver>,
    pub management_client: Option<Arc<Mutex<ZitadelManagementClient>>>,
    pub issuer_url: String,
}

impl UserService {
    pub fn new(
        user_resolver: Arc<UserResolver>,
        management_client: Option<Arc<Mutex<ZitadelManagementClient>>>,
        issuer_url: String,
    ) -> Self {
        Self {
            user_resolver,
            management_client,
            issuer_url,
        }
    }

    pub async fn get_user_by_identity(
        &self,
        issuer: &str,
        subject: &str,
    ) -> Result<Option<Arc<UserEntity>>, Error> {
        self.user_resolver.find_and_cache_user_by_identity(issuer, subject).await
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

        self.user_resolver
            .upsert_and_cache_user(issuer, sub, &username, &email)
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
            let mut client = client.lock().await;
            let users = client.list_all_users().await?;

            let mut synced_count = 0;
            let mut error_count = 0;

            // Sync each user to database
            for (issuer, subject, username, email) in users {
                match self.user_resolver
                    .upsert_and_cache_user(&issuer, &subject, &username, &email)
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
                users.into_iter().map(|(_, s, _, _)| s).collect();
            
            let db_identities = self.user_resolver.get_all_identities().await?;
            for (db_issuer, db_subject) in db_identities {
                if db_issuer == self.issuer_url && !zitadel_subjects.contains(&db_subject) {
                    tracing::warn!("Removing user {} - no longer in ZITADEL", db_subject);
                    self.user_resolver.remove_user_from_cache_and_db(&db_issuer, &db_subject).await?;
                }
            }
            */
        } else {
            // Just refresh cache from DB if no management client
            tracing::info!("No ZITADEL Management client - refreshing cache from database");
            self.user_resolver.refresh_cache().await?;
        }

        tracing::info!(
            "User cache contains {} users", 
            self.user_resolver.cache.entry_count()
        );

        Ok(())
    }
}