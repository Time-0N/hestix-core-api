#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct OidcClaims {
    pub exp: u64,
    pub iat: u64,
    pub iss: String,
    pub aud: String,
    pub sub: String,
    pub email: Option<String>,
    pub preferred_username: Option<String>,
    pub roles: Vec<String>,
}

