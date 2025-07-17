use crate::dto::auth::token_response::TokenResponse;
use crate::dto::keycloak::keycloak_user::KeycloakUser;
use crate::security::keycloak::claims::KeycloakClaims;
use crate::security::keycloak::client::KeycloakClient;
use crate::security::keycloak::KeycloakError;
use crate::security::keycloak::validator::validate_token_and_extract_claims;

#[derive(Clone)]
pub struct KeycloakService {
    pub client: KeycloakClient,
}

impl KeycloakService {
    pub fn new(client: KeycloakClient) -> Self {
        Self { client }
    }
    
    pub async fn fetch_user_token(&self, username: &str, password: &str) -> Result<TokenResponse, KeycloakError> {
        self.client.fetch_user_token(username, password).await
    }

    pub async fn validate_token(&self, token: &str) -> Result<KeycloakClaims, KeycloakError> {
        validate_token_and_extract_claims(token).await
    }

    pub async fn fetch_all_users(&self) -> Result<Vec<KeycloakUser>, KeycloakError> {
        self.client.fetch_all_users().await
    }
    
    pub async fn check_health(&self) -> bool {
        self.client.check_health().await
    }

    async fn fetch_admin_token(&self) -> Result<String, KeycloakError> {
        self.client.fetch_admin_token().await
    }
}