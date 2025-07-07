use axum::{Router, routing::post};
use crate::handlers::auth_handler::login;
use crate::app_state::AppState;

pub fn auth_routes(state: AppState) -> Router {
    Router::new()
        .route("/login", post(login))
        .with_state(state)
}
