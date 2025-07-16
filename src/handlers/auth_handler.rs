use std::sync::Arc;
use axum::{Extension, Json};
use reqwest::StatusCode;
use crate::dto::auth::login_request::LoginRequest;
use crate::dto::auth::token_response::TokenResponse;
use crate::services::auth_service::AuthService;

pub async fn login_user_handler(
    Extension(svc): Extension<Arc<AuthService>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<TokenResponse>, (StatusCode, String)> {
    svc
        .login_user(payload)
        .await
        .map(Json)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))
}