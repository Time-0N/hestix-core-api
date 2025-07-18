use axum::{Router};
use axum::routing::{get};
use crate::app_state::AppState;
use crate::handlers::auth_handler::{login_handler, logout_handler, oauth_callback_handler, refresh_handler};

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/login",    get(login_handler))
        .route("/logout",   get(logout_handler))
        .route("/refresh",  get(refresh_handler))
        .route("/callback", get(oauth_callback_handler))
}