use reqwest::Client;
use crate::dto::auth::token_response::TokenResponse;
use crate::security::keycloak::config::KeycloakConfig;
use crate::dto::keycloak::keycloak_user_create::KeycloakUserCreate;
use crate::security::keycloak::KeycloakError;

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

    pub async fn create_user(&self, user: &KeycloakUserCreate, token: &str) -> Result<(), KeycloakError> {
        let url = format!(
            "{}/admin/realms/{}/users",
            self.config.base_url, self.config.realm
        );

        let res = self.client
            .post(url)
            .bearer_auth(token)
            .json(user)
            .send()
            .await?;

        match res.status().as_u16() {
            201 => Ok(()),
            409 => Err(KeycloakError::UserAlreadyExists),
            401 => Err(KeycloakError::Unauthorized),
            _ => {
                let text = res.text().await.unwrap_or_else(|_| "Unknown".into());
                Err(KeycloakError::UnexpectedResponse(text))
            }
        }


    }
}