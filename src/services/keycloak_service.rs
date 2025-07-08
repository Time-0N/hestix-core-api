use crate::dto::auth::RegisterUserRequest;
use crate::security::keycloak::client::KeycloakClient;
use crate::dto::keycloak::keycloak_user_create::KeycloakUserCreate;
use crate::security::keycloak::KeycloakError;

#[derive(Clone)]
pub struct KeycloakService {
    pub client: KeycloakClient,
}

impl KeycloakService {
    pub fn new(client: KeycloakClient) -> Self {
        Self { client }
    }

    pub async fn register_user(
        &self,
        req: RegisterUserRequest,
        token: &str,
    ) -> Result<(), KeycloakError> {
        let user = KeycloakUserCreate::new(req.username, req.email, req.password);
        let result = self.client.create_user(&user, token).await;

        match result {
            Ok(_) => Ok(()),
            Err(KeycloakError::UserAlreadyExists) => Ok(()),
            Err(e) => {
                tracing::error!("Keycloak error: {:?}", e);
                Err(e)
            }
        }
    }

    pub async fn fetch_admin_token(&self) -> Result<String, KeycloakError> {
        self.client.fetch_admin_token().await
    }
}