use axum::extract::State;
use axum::Json;
use reqwest::StatusCode;
use crate::app_state::AppState;
use crate::dto::auth::login_request::LoginRequest;
use crate::dto::auth::RegisterUserRequest;
use crate::dto::auth::token_response::TokenResponse;

pub async fn register_user_handler(
    State(state): State<AppState>,
    Json(payload): Json<RegisterUserRequest>
) -> Result<(), (StatusCode, String)> {
    state.auth_service
        .register_user(payload)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn login_user_handler(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<TokenResponse>, (StatusCode, String)> {
    state.auth_service
        .login_user(payload)
        .await
        .map(Json)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))
}