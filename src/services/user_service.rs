use std::collections::HashSet;
use std::sync::Arc;
use anyhow::Context;
use sqlx::{Error};
use time::OffsetDateTime;
use uuid::Uuid;
use crate::cache::user_resolver::UserResolver;
use crate::dto::keycloak::keycloak_user::KeycloakUser;
use crate::models::user::UserEntity;
use crate::security::keycloak::claims::KeycloakClaims;
use crate::security::keycloak::KeycloakError;
use crate::services::keycloak_service::KeycloakService;

#[derive(Clone)]
pub struct UserService {
    pub user_resolver: Arc<UserResolver>,
    pub keycloak_service: Arc<KeycloakService>
}

impl UserService {
    pub fn new(user_resolver: Arc<UserResolver>, keycloak_service: Arc<KeycloakService> ) -> Self {
        Self { user_resolver, keycloak_service }
    }

    pub async fn get_user_by_keycloak_id(&self, keycloak_id: Uuid) -> Result<Option<Arc<UserEntity>>, Error> {
        self.user_resolver.resolver_by_keycloak_id(keycloak_id).await
    }

    pub async fn sync_user_from_keycloak_claims(
        &self,
        claims: &KeycloakClaims,
    ) -> Result<(), KeycloakError> {
        let sub = claims
            .sub
            .as_ref()
            .ok_or_else(|| KeycloakError::Other("Missing `sub` claim in JWT".to_string()))?;

        let keycloak_id = Uuid::parse_str(sub)
            .map_err(|e| KeycloakError::Other(format!("Invalid UUID in sub claim: {}", e)))?;

        let username = claims
            .preferred_username
            .as_ref()
            .ok_or_else(|| KeycloakError::Other("Missing `preferred_username` in claims".to_string()))?
            .clone();

        let existing_user = self
            .user_resolver
            .resolver_by_keycloak_id(keycloak_id)
            .await
            .map_err(|e| KeycloakError::Other(format!("User resolver failed: {}", e)))?;

        if existing_user.is_none() {
            let new_user = UserEntity {
                id: Uuid::new_v4(),
                keycloak_id,
                username,
                email: claims.email.clone().unwrap_or_default(),
                created_at: OffsetDateTime::now_utc(),
                updated_at: OffsetDateTime::now_utc(),

            };

            self.user_resolver
                .insert_and_cache_user(new_user)
                .await
                .map_err(|e| KeycloakError::Other(format!("User insert failed: {}", e)))?;
        }

        Ok(())
    }

    pub async fn sync_users(&self) -> anyhow::Result<()> {
        let local_user_ids: HashSet<Uuid> = self
            .user_resolver
            .get_all_user_ids()
            .await?
            .into_iter()
            .collect();

        let remote_users: Vec<KeycloakUser> = self
            .keycloak_service
            .fetch_all_users()
            .await
            .context("Failed to fetch users from Keycloak")?;
        
        let remote_user_ids: HashSet<Uuid> = remote_users
            .into_iter()
            .map(|user| user.id)
            .collect();
        
        for orphan_id in local_user_ids.difference(&remote_user_ids) {
            self.user_resolver
                .remove_user_from_cache_and_db(*orphan_id)
                .await
                .with_context(|| format!("Failed to delete orphaned user: {}", orphan_id))?;
            
            tracing::info!("Deleted orphaned user: {}", orphan_id);
        }
        
        tracing::info!(
            "User sync complete - Local: {}, Remote: {}, Deleted: {}",
            local_user_ids.len(),
            remote_user_ids.len(),
            local_user_ids.difference(&remote_user_ids).count()
        );
        
        Ok(())
    }

}