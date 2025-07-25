use axum::{Json};
use axum::extract::State;
use axum::http::StatusCode;
use uuid::Uuid;
use crate::app_state::AppState;
use crate::dto::user::user_response::UserResponse;
use crate::middleware::security::keycloak::extractor::Claims;
use crate::require_role;

pub async fn get_user_info(
    State(state): State<AppState>,
    Claims(claims): Claims,
) -> Result<Json<UserResponse>, (StatusCode, String)> {
    require_role!(claims, "user");

    let sub = claims.sub
        .as_deref()
        .ok_or((StatusCode::UNAUTHORIZED, "Missing `sub` in token".into()))?;

    let keycloak_id = Uuid::parse_str(sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid UUID in `sub`".into()))?;

    let user = state
        .user_service
        .get_user_by_keycloak_id(keycloak_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "User not found".into()))?;

    let response = UserResponse::from((user, &claims));

    Ok(Json(response))
}
