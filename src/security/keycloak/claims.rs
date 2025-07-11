use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct KeycloakClaims {
    pub exp: u64,
    pub iat: u64,
    pub iss: String,
    pub aud: String,
    pub sub: Option<String>,
    pub email: Option<String>,
    pub preferred_username: Option<String>,
    pub realm_access: RealmAccess,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RealmAccess {
    pub roles: Vec<String>,
}