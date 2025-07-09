use axum::{Router, routing::get};
use crate::handlers::user_handler::get_user_info;
use crate::app_state::AppState;

pub fn user_routes(state: AppState) -> Router {
    Router::new()
        .route("/find/{id}", get(get_user_info))
        .with_state(state)
}
