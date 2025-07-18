pub mod auth_routes;
pub mod user_routes;

use axum::{middleware, Router};
use crate::app_state::AppState;
use crate::middleware::cookies::propagate_cookies;
use crate::routes::auth_routes::auth_routes;
use crate::routes::user_routes::user_routes;

pub fn create_router() -> Router<AppState> {
    Router::new()
        .nest("/api/user", user_routes())
        .nest("/api/auth", auth_routes())
        .layer(middleware::from_fn(propagate_cookies))
}
