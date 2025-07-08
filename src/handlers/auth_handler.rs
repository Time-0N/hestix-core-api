use axum::Router;
use axum::routing::post;
use crate::app_state::AppState;
use crate::routes::auth_routes::register_user_handler;

pub fn auth_routes(state: AppState) -> Router {
    Router::new()
        .route("/register", post(register_user_handler))
        .with_state(state)
}