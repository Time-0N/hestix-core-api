use std::sync::Arc;
use crate::dto::auth::token_response::TokenResponse;
use crate::middleware::security::keycloak::KeycloakError;
use crate::services::keycloak_service::KeycloakService;
use crate::services::user_service::UserService;

#[derive(Clone)]
pub struct AuthService {
    pub keycloak_service: Arc<KeycloakService>,
    user_service: Arc<UserService>,
}

impl AuthService {
    pub fn new(keycloak_service: Arc<KeycloakService>, user_service: Arc<UserService>) -> Self {
        Self { keycloak_service, user_service }
    }

    pub async fn exchange_code_for_token(&self, code: String) -> Result<TokenResponse, KeycloakError> {
        let token = self.keycloak_service
            .exchange_code_for_token(&code)
            .await?;

        let claims = self.keycloak_service
            .validate_token(&token.access_token)
            .await?;

        self.user_service.sync_user_from_keycloak_claims(&claims).await?;

        Ok(token)
    }

    pub async fn refresh_access_token(&self, refresh_token: &str) -> Result<TokenResponse, KeycloakError> {
        self.keycloak_service.refresh_access_token(refresh_token).await
    }

    pub fn build_authorize_url(&self) -> String {
        format!(
            "{}/realms/{}/protocol/openid-connect/auth?client_id={}&response_type=code&redirect_uri={}&scope=openid",
            self.keycloak_service.client.config.base_url,
            self.keycloak_service.client.config.realm,
            self.keycloak_service.client.config.client_id,
            self.keycloak_service.client.config.redirect_uri
        )
    }

}