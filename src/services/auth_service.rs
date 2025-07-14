use std::sync::Arc;
use crate::dto::auth::login_request::LoginRequest;
use crate::dto::auth::RegisterUserRequest;
use crate::dto::auth::token_response::TokenResponse;
use crate::security::keycloak::KeycloakError;
use crate::services::keycloak_service::KeycloakService;
use crate::user::user_service::UserService;

#[derive(Clone)]
pub struct AuthService {
    pub keycloak_service: Arc<KeycloakService>,
    user_service: Arc<UserService>,
}

impl AuthService {
    pub fn new(keycloak_service: Arc<KeycloakService>, user_service: Arc<UserService>) -> Self {
        Self { keycloak_service, user_service }
    }

    pub async fn register_user(
        &self,
        req: RegisterUserRequest,
    ) -> Result<(), KeycloakError> {
        let keycloak_id = self.keycloak_service
            .create_keycloak_user(req.clone())
            .await?;

        self.user_service
            .create_user(req, keycloak_id.parse().unwrap())
            .await
            .map_err(|e| KeycloakError::Other(e.to_string()))
    }
    
    pub async fn login_user(&self, request: LoginRequest) -> Result<TokenResponse, KeycloakError> {
        self.keycloak_service
            .fetch_user_token(&request.username, &request.password)
            .await
    }

}