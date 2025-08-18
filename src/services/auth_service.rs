use std::sync::Arc;
use crate::dto::auth::token_response::TokenResponse;
use crate::services::user_service::UserService;
use crate::oidc::{OidcClaims};
use crate::oidc::provider::OidcProvider;
use crate::oidc::error::OidcError;

#[derive(Clone)]
pub struct AuthService {
    pub provider: Arc<dyn OidcProvider + Send + Sync>,
    user_service: Arc<UserService>,
}

impl AuthService {
    pub fn new(provider: Arc<dyn OidcProvider + Send + Sync>, user_service: Arc<UserService>) -> Self {
        Self { provider, user_service }
    }

    pub async fn exchange_code_for_token(
        &self,
        code: String,
        code_verifier: Option<String>,
    ) -> Result<TokenResponse, OidcError> {
        let tokens = self.provider.exchange_code_for_tokens(&code, code_verifier.as_deref()).await?;

        // Always validate access token to get roles + base checks
        let mut access_claims = self.provider.validate_access_token(&tokens.access_token).await?;

        // If we received an id_token and the toggle is enabled, validate and enrich
        if let Some(idt) = &tokens.id_token {
            if let Ok(id_claims) = self.provider.validate_id_token(idt).await {
                // overwrite/enrich user-facing fields from ID token
                if access_claims.email.is_none() {
                    access_claims.email = id_claims.email;
                }
                if access_claims.preferred_username.is_none() {
                    access_claims.preferred_username = id_claims.preferred_username;
                }
                // Note: keep roles from access token; ID token typically doesn't carry them
            }
        } else {
            // (Optional fallback) If you didn’t enable the toggle, you could still call userinfo here.
            // let ui = self.provider.fetch_userinfo(&tokens.access_token).await?;
            // ...enrich access_claims from ui...
        }

        // Persist user
        self.user_service.sync_user_from_claims(&access_claims).await?;

        Ok(tokens)
    }


    pub async fn refresh_access_token(&self, refresh_token: &str) -> Result<TokenResponse, OidcError> {
        self.provider.refresh_access_token(refresh_token).await
    }

    pub async fn validate(&self, token: &str) -> Result<OidcClaims, OidcError> {
        self.provider.validate_access_token(token).await
    }

    pub async fn build_authorize_url(&self, code_challenge: Option<&str>, state: Option<String>) -> String {
        self.provider.authorize_url(state, code_challenge).await
    }
}
