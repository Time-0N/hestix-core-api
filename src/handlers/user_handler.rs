use axum::{extract::{ State}, Json};
use axum::http::StatusCode;
use uuid::Uuid;
use crate::app_state::AppState;
use crate::models::user::User;
use crate::security::keycloak::extractor::Claims;

pub async fn get_user_info(
    State(state): State<AppState>,
    Claims(claims): Claims,
) -> Result<Json<User>, (StatusCode, String)> {
    let sub = claims.sub
        .as_deref()
        .ok_or((StatusCode::UNAUTHORIZED, "Missing `sub` in token".into()))?;

    let keycloak_id = Uuid::parse_str(sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid UUID in `sub`".into()))?;

    match state.user_service.get_user_by_keycloak_id(keycloak_id).await {
        Ok(Some(user)) => Ok(Json((*user).clone())),
        Ok(None) => Err((StatusCode::NOT_FOUND, "User not found".into())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}
