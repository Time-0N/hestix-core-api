use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct UserLoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct UserLoginResponse {
    pub token: String,
    pub expires_in: u64,
}