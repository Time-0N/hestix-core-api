use std::sync::Arc;
use axum::{Extension, Router};
use axum::routing::{get};
use crate::handlers::auth_handler::oauth_callback_handler;
use crate::services::auth_service::AuthService;

pub fn auth_routes(auth_service: Arc<AuthService>) -> Router {
    Router::new()
        .route("/callback", get(oauth_callback_handler))
        .layer(Extension(auth_service))
}