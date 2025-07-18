use crate::dto::auth::token_response::TokenResponse;
use crate::dto::keycloak::keycloak_user::KeycloakUser;
use reqwest::Client;
use crate::middleware::security::keycloak::config::KeycloakConfig;
use crate::middleware::security::keycloak::KeycloakError;

#[derive(Clone)]
pub struct KeycloakClient {
    pub config: KeycloakConfig,
    pub client: Client,
}

impl KeycloakClient {
    pub fn new(config: KeycloakConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    pub async fn fetch_admin_token(&self) -> Result<String, KeycloakError> {
        let url = format!(
            "{}/realms/{}/protocol/openid-connect/token",
            self.config.base_url, self.config.realm
        );

        let res = self.client
            .post(url)
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", &self.config.client_id),
                ("client_secret", &self.config.client_secret),
            ])
            .send()
            .await?
            .error_for_status()?;

        let token_response: TokenResponse = res.json().await.map_err(|_| KeycloakError::MissingToken)?;
        Ok(token_response.access_token)
    }

    pub async fn exchange_code_for_token(&self, code: &str) -> Result<TokenResponse, KeycloakError> {
        let url = format!("{}/realms/{}/protocol/openid-connect/token", self.config.base_url, self.config.realm);

        let res = self.client
            .post(url)
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("redirect_uri", &self.config.redirect_uri),
                ("client_id", &self.config.client_id),
                ("client_secret", &self.config.client_secret),
            ])
            .send()
            .await?
            .error_for_status()?;

        let token: TokenResponse = res.json().await.map_err(|_| KeycloakError::MissingToken)?;
        Ok(token)
    }
    
    pub async fn fetch_all_users(&self) -> Result<Vec<KeycloakUser>, KeycloakError> {
        let admin_token = self.fetch_admin_token().await?;
        let mut all_users = Vec::new();
        let mut first = 0;
        let max = 100;

        loop {
            let url = format!(
                "{}/admin/realms/{}/users?first={}&max={}",
                self.config.base_url,
                self.config.realm,
                first,
                max
            );

            let response = self.client
                .get(&url)
                .bearer_auth(&admin_token)
                .send()
                .await?
                .error_for_status()?
                .json::<Vec<KeycloakUser>>()
                .await?;

            if response.is_empty() {
                break;
            }

            all_users.extend(response);
            first += max;
        }

        Ok(all_users)
    }

    pub async fn refresh_access_token(&self, refresh_token: &str) -> Result<TokenResponse, KeycloakError> {
        let url = format!(
            "{}/realms/{}/protocol/openid-connect/token",
            self.config.base_url,
            self.config.realm
        );

        let res = self.client
            .post(url)
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
                ("client_id", &self.config.client_id),
                ("client_secret", &self.config.client_secret),
            ])
            .send()
            .await?
            .error_for_status()?;

        let token: TokenResponse = res.json().await.map_err(|_| KeycloakError::MissingToken)?;
        Ok(token)
    }

    pub async fn check_health(&self) -> bool {
        let url = format!("{}/realms/{}", self.config.base_url, self.config.realm);

        match self.client.get(&url).send().await {
            Ok(response) if response.status().is_success() => {
                tracing::info!("Keycloak health check successful at {}", url);
                true
            }
            Ok(response) => {
                tracing::warn!("⚠Keycloak health check returned non-success status: {}", response.status());
                false
            }
            Err(e) => {
                tracing::error!("Keycloak health check failed: {:?}", e);
                false
            }
        }
    }
}