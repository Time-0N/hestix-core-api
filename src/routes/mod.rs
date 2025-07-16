pub mod auth_routes;
pub mod user_routes;

use axum::Router;
use crate::app_state::AppState;
use crate::routes::auth_routes::auth_routes;
use crate::routes::user_routes::user_routes;

pub fn create_router(state: AppState) -> Router {
    let AppState { user_service, auth_service, .. } = state;

    Router::new()
        .nest("/api/cache", user_routes(user_service))
        .nest("/api/auth", auth_routes(auth_service))
}
