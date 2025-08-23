use std::sync::Arc;
use serde::Serialize;
use uuid::Uuid;
use crate::util::oidc::claims::OidcClaims;
use crate::model::user::UserEntity;

#[derive(Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
}

impl From<(Arc<UserEntity>, &OidcClaims)> for UserResponse {
    fn from((user, claims): (Arc<UserEntity>, &OidcClaims)) -> Self {
        UserResponse {
            id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            roles: claims.roles.clone(),
        }
    }
}
