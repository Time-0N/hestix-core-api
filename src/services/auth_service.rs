use crate::dto::auth::RegisterUserRequest;
use crate::security::keycloak::KeycloakError;
use crate::services::keycloak_service::KeycloakService;
use crate::services::user_service::UserService;

pub struct AuthService {
    pub keycloak_service: KeycloakService,
    pub user_service: UserService,
}

impl AuthService {
    pub fn new(keycloak_service: KeycloakService, user_service: UserService) -> Self {
        Self { keycloak_service, user_service }
    }

    pub async fn register_user(
        &self,
        req: RegisterUserRequest,
    ) -> Result<(), KeycloakError> {
        let token = self.keycloak_service.fetch_admin_token().await?;

        self.keycloak_service
            .register_user(req.clone(), &token)
            .await?;

        self.user_service
            .register_user(req)
            .await
            .map_err(|e| KeycloakError::Other(e.to_string()))
    }

}