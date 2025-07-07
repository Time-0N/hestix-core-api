use axum::{extract::State, Json};
use crate::app_state::AppState;
use serde::Serialize;

#[derive(Serialize)]
pub struct DummyUserResponse {
    pub id: u32,
    pub username: String,
    pub email: String,
}

pub async fn get_user_info(State(_state): State<AppState>) -> Json<DummyUserResponse> {
    // Dummy hardcoded user
    Json(DummyUserResponse {
        id: 1,
        username: "dummy_user".to_string(),
        email: "dummy@example.com".to_string(),
    })
}