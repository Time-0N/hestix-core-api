use axum::{Router, routing::get};
use crate::app_state::AppState;
use crate::infrastructure::web::handlers::user_handler::get_user_info;

pub fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/me", get(get_user_info))
}
