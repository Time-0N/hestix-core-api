use axum::Router;
use axum::routing::post;
use crate::app_state::AppState;
use crate::handlers::auth_handler::{login_user_handler, register_user_handler};

pub fn auth_routes(state: AppState) -> Router {
    Router::new()
        .route("/register", post(register_user_handler))
        .route("/login", post(login_user_handler))
        .with_state(state)
}