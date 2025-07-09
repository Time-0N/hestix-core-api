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
    pub async fn create_keycloak_user(
        &self,
        req: RegisterUserRequest,
    ) -> Result<String, KeycloakError> {
        let token = self.fetch_admin_token().await?;
        let user = KeycloakUserCreate::new(req.username, req.email, req.password);

        match self.client.create_user(&user, &token).await {
            Ok(id) => Ok(id),
            Err(KeycloakError::UserAlreadyExists) => {
                tracing::warn!("Tried to register a user that already exists: {}", user.email);
                Err(KeycloakError::UserAlreadyExists)
            }
            Err(e) => {
                tracing::error!("Keycloak error: {:?}", e);
                Err(e)
            }
        }
    }
    
    pub async fn check_health(&self) -> bool {
        self.client.check_health().await
    }

    async fn fetch_admin_token(&self) -> Result<String, KeycloakError> {
        self.client.fetch_admin_token().await
    }
}