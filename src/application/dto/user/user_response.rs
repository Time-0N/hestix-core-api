use std::sync::Arc;
use serde::Serialize;
use uuid::Uuid;
use crate::infrastructure::oidc::claims::OidcClaims;
use crate::domain::entities::User;

#[derive(Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
}

impl From<(Arc<User>, &OidcClaims)> for UserResponse {
    fn from((user, claims): (Arc<User>, &OidcClaims)) -> Self {
        UserResponse {
            id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            roles: claims.roles.clone(),
        }
    }
}
