use axum::extract::State;
use axum::Json;
use reqwest::StatusCode;
use crate::app_state::AppState;
use crate::dto::auth::RegisterUserRequest;

pub async fn register_user_handler(
    State(state): State<AppState>,
    Json(payload): Json<RegisterUserRequest>
) -> Result<(), (StatusCode, String)> {
    state.auth_service
        .register_user(payload)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}
