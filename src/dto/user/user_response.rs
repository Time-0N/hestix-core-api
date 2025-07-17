use std::sync::Arc;
use serde::Serialize;
use uuid::Uuid;
use crate::middleware::security::keycloak::claims::KeycloakClaims;
use crate::models::user::UserEntity;

#[derive(Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
}

impl From<(Arc<UserEntity>, &KeycloakClaims)> for UserResponse {
    fn from((user, claims): (Arc<UserEntity>, &KeycloakClaims)) -> Self {
        UserResponse {
            id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            roles: claims.realm_access.roles.clone(),
        }
    }
}
