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

    pub async fn exchange_code_for_token(&self, code: String, code_verifier: String) -> Result<TokenResponse, KeycloakError> {
        let token = self.keycloak_service
            .exchange_code_for_token(&code, &code_verifier)
            .await?;

        let claims = self.keycloak_service
            .validate_token(&token.access_token)
            .await?;
        
        self.user_service.sync_user_from_keycloak_claims(&claims).await?;
        
        Ok(token)
    }

}