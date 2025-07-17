use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct KeycloakUser {
    pub id: Uuid,
    pub username: Option<String>,
    pub email: Option<String>,
}