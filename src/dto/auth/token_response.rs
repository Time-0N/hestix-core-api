use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct TokenResponse {
    pub access_token: String,
    pub expires_in: Option<i64>,
    pub refresh_expires_in: Option<i64>,
    pub refresh_token: Option<String>,
    pub token_type: Option<String>,
    pub not_before_policy: Option<i64>,
    pub session_state: Option<String>,
    pub scope: Option<String>,
}