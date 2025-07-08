use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct RegisterUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}