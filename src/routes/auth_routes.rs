use std::sync::Arc;
use axum::{Extension, Router};
use axum::routing::post;
use crate::handlers::auth_handler::{login_user_handler};
use crate::services::auth_service::AuthService;

pub fn auth_routes(auth_service: Arc<AuthService>) -> Router {
    Router::new()
        .route("/login", post(login_user_handler))
        .layer(Extension(auth_service))
}