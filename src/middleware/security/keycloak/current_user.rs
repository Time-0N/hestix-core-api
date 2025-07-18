use std::sync::Arc;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    middleware::security::keycloak::extractor::Claims,
    models::user::UserEntity,
};

pub struct CurrentUser(pub Arc<UserEntity>);

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync + 'static,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let state = parts
            .extensions
            .get::<AppState>()
            .cloned()
            .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Missing AppState"))?;

        let Claims(claims) = Claims::from_request_parts(parts, &state)
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid claims"))?;

        let sub = claims.sub.as_deref()
            .ok_or((StatusCode::UNAUTHORIZED, "Missing sub"))?;

        let uuid = Uuid::parse_str(sub)
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid UUID"))?;

        let user = state.user_service.get_user_by_keycloak_id(uuid)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "DB error"))?
            .ok_or((StatusCode::NOT_FOUND, "User not found"))?;

        Ok(CurrentUser(user))
    }
}
