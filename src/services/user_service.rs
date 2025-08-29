use std::sync::Arc;
use sqlx::Error;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::util::cache::user_resolver::UserResolver;
use crate::model::user::UserEntity;
use crate::util::oidc::{OidcClaims, OidcError};
use crate::util::oidc::provider::OidcProvider;

#[derive(Clone)]
pub struct UserService {
    pub user_resolver: Arc<UserResolver>,
    pub oidc_provider: Arc<dyn OidcProvider>,
}

impl UserService {
    pub fn new(
        user_resolver: Arc<UserResolver>,
        oidc_provider: Arc<dyn OidcProvider>
    ) -> Self {
        Self {
            user_resolver,
            oidc_provider
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

    // Optional: full sync if you later add admin API support for Zitadel
    pub async fn sync_users(&self) -> anyhow::Result<()> {
        tracing::info!("Starting user sync from ZITADEL");

        // For ZITADEL, you'll need to implement the admin API
        // For now, let's at least refresh the cache with DB users
        self.user_resolver.refresh_cache().await?;

        // TODO: When you implement ZITADEL admin API:
        // 1. Fetch all users from ZITADEL via Management API
        // 2. For each user, upsert into database
        // 3. Remove users that no longer exist in ZITADEL

        tracing::info!("User sync completed - cache refreshed with {} users",
            self.user_resolver.cache.entry_count());

        Ok(())
    }
}
