use axum::{extract::{Path, State}, Json};
use uuid::Uuid;
use crate::app_state::AppState;
use crate::models::user::User;

pub async fn get_user_info(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<User>, (axum::http::StatusCode, String)> {
    match state.user_service.get_user_by_id(user_id).await {
        Ok(Some(user)) => Ok(Json(user)),
        Ok(None) => Err((axum::http::StatusCode::NOT_FOUND, "User not found".into())),
        Err(e) => Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}