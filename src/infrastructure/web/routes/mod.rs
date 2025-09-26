pub mod auth_routes;
pub mod user_routes;

use axum::{middleware, Router};
use crate::app_state::AppState;
use crate::shared::middleware::cookies::propagate_cookies_middleware;
use crate::infrastructure::web::routes::auth_routes::auth_routes;
use crate::infrastructure::web::routes::user_routes::user_routes;

pub fn create_router() -> Router<AppState> {
    Router::new()
        .nest("/api/user", user_routes())
        .nest("/api/auth", auth_routes())
        .layer(middleware::from_fn(propagate_cookies_middleware))
}
