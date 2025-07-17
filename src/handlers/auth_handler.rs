use std::sync::Arc;
use axum::{Extension, Json};
use axum::extract::Query;
use reqwest::StatusCode;
use crate::dto::auth::auth_callback_request::AuthCallbackRequest;
use crate::dto::auth::token_response::TokenResponse;
use crate::services::auth_service::AuthService;

pub async fn oauth_callback_handler(
    Query(query): Query<AuthCallbackRequest>,
    Extension(svc): Extension<Arc<AuthService>>,
) -> Result<Json<TokenResponse>, (StatusCode, String)> {
    svc.exchange_code_for_token(query.code, query.code_verifier).await
        .map(Json)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Auth failed: {e}")))
}