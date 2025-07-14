use std::sync::Arc;
use crate::dto::auth::login_request::LoginRequest;
use crate::dto::auth::token_response::TokenResponse;
use crate::security::keycloak::KeycloakError;
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

    pub async fn login_user(&self, request: LoginRequest) -> Result<TokenResponse, KeycloakError> {
        let token = self.keycloak_service
            .fetch_user_token(&request.username, &request.password)
            .await?;

        let claims = self.keycloak_service
            .validate_token(&token.access_token)
            .await?;

        self.user_service.sync_user_from_keycloak_claims(&claims).await?;

        Ok(token)
    }

}