use axum::{Json, extract::State};
use crate::app_state::AppState;
use crate::dto::user_dto::{UserLoginRequest, UserLoginResponse};

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<UserLoginRequest>
) -> Json<UserLoginResponse> {
    // Mock response for now
    Json(UserLoginResponse {
        token: "mock".to_string(),
        expires_in: 3600,
    })
}
