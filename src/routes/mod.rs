pub mod auth_routes;
pub mod user_routes;

use axum::Router;
use crate::app_state::AppState;
use crate::routes::auth_routes::auth_routes;
use crate::routes::user_routes::user_routes;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .nest("/api/user", user_routes(state.clone()))
        .nest("/api/auth", auth_routes(state))
}
