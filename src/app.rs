use axum::Router;

use crate::routes::user::user_routes;

pub fn create_app() -> Router {
    Router::new().merge(user_routes())
}