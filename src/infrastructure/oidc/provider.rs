use crate::application::dto::auth::token_response::TokenResponse;
use crate::infrastructure::oidc::claims::OidcClaims;
use crate::infrastructure::oidc::error::OidcError;

#[async_trait::async_trait]
pub trait OidcProvider: Send + Sync {
    /// Build an authorization URL. If `code_challenge` is Some, PKCE S256 is used.
    async fn authorize_url(&self, state: Option<String>, code_challenge: Option<&str>) -> String;

    /// Exchange the authorization code for tokens. If `code_verifier` is Some, PKCE is used.
    async fn exchange_code_for_tokens(&self, code: &str, code_verifier: Option<&str>) -> Result<TokenResponse, OidcError>;

    async fn refresh_access_token(&self, refresh_token: &str) -> Result<TokenResponse, OidcError>;

    async fn validate_access_token(&self, token: &str) -> Result<OidcClaims, OidcError>;

    async fn validate_id_token(&self, id_token: &str) -> Result<OidcClaims, OidcError>;

    /// Revoke a token at the provider (for proper logout)
    async fn revoke_token(&self, token: &str) -> Result<(), OidcError>;
}

#[async_trait::async_trait]
pub trait RoleMapper: Send + Sync {
    fn extract_roles(&self, raw_claims: &serde_json::Value) -> Vec<String>;
}

#[async_trait::async_trait]
pub trait OidcAdminApi: Send + Sync {
    async fn fetch_all_users(&self) -> Result<Vec<IdpUser>, OidcError>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IdpUser {
    pub idp_subject: String,
    pub username: Option<String>,
    pub email: Option<String>
}
