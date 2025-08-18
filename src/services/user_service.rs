use std::sync::Arc;
use sqlx::Error;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::cache::user_resolver::UserResolver;
use crate::models::user::UserEntity;
use crate::oidc::{OidcClaims, OidcError};

#[derive(Clone)]
pub struct UserService {
    pub user_resolver: Arc<UserResolver>,
    // optional: admin api later
    // pub admin_api: Option<Arc<dyn OidcAdminApi + Send + Sync>>,
}

impl UserService {
    pub fn new(user_resolver: Arc<UserResolver>, _admin_api: Option<()>) -> Self {
        Self { user_resolver }
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

        let email = claims.email.clone().unwrap_or_default();

        let existing = self
            .get_user_by_identity(issuer, sub)
            .await
            .map_err(|e| OidcError::Provider(format!("User resolver failed: {}", e)))?;

        if existing.is_none() {
            let now = OffsetDateTime::now_utc();
            let new_user = UserEntity {
                id: Uuid::new_v4(),
                idp_issuer: issuer.clone(),
                idp_subject: sub.clone(),
                username,
                email,
                created_at: now,
                updated_at: now,
            };

            self.user_resolver
                .insert_and_cache_user(new_user)
                .await
                .map_err(|e| OidcError::Provider(format!("User insert failed: {}", e)))?;
        }

        Ok(())
    }

    // Optional: full sync if you later add admin API support for Zitadel
    pub async fn sync_users(&self) -> anyhow::Result<()> {
        // placeholder: no-op until admin API implemented
        Ok(())
    }
}
