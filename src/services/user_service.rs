use std::sync::Arc;
use sqlx::{Error};
use time::OffsetDateTime;
use uuid::Uuid;
use crate::cache::resolver::UserResolver;
use crate::models::user::UserEntity;
use crate::security::keycloak::claims::KeycloakClaims;
use crate::security::keycloak::KeycloakError;

#[derive(Clone)]
pub struct UserService {
    pub user_resolver: Arc<UserResolver>,
}

impl UserService {
    pub fn new(user_resolver: Arc<UserResolver> ) -> Self {
        Self { user_resolver }
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

}