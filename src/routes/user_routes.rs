use std::sync::Arc;
use axum::{Router, routing::get, Extension};
use crate::handlers::user_handler::get_user_info;
use crate::services::user_service::UserService;

pub fn user_routes(user_service: Arc<UserService>) -> Router {
    Router::new()
        .route("/me", get(get_user_info))
        .layer(Extension(user_service))
}
