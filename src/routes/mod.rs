pub mod auth_routes;
pub mod user_routes;

use axum::Router;
use crate::app_state::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .nest("/api/auth", auth_routes::auth_routes(state.clone()))
        .nest("/api/users", user_routes::user_routes(state))
}
