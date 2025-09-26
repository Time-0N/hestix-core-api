use axum::{Json};
use axum::extract::State;
use axum::http::StatusCode;
use crate::app_state::AppState;
use crate::application::dto::user::user_response::UserResponse;
use crate::shared::middleware::security::extractor::Claims;
use crate::require_role;
use crate::infrastructure::web::errors::server_fail;

pub async fn get_user_info(
    State(state): State<AppState>,
    Claims(claims): Claims,
) -> Result<Json<UserResponse>, (StatusCode, String)> {
    require_role!(claims, "user");

    // issuer + subject (strings)
    let issuer = &claims.iss;
    let subject = &claims.sub;

    let user = state
        .user_service
        .get_user_by_identity(issuer, subject)
        .await
        .map_err(server_fail("get_user_by_identity"))?
        .ok_or((StatusCode::NOT_FOUND, "User not found".into()))?;

    let response = UserResponse::from((user, &claims));
    Ok(Json(response))
}
