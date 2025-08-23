use axum::{routing::{get, post}, Router};
use http::header::{CACHE_CONTROL, PRAGMA};
use http::HeaderValue;
use tower_http::set_header::SetResponseHeaderLayer;
use crate::app_state::AppState;
use crate::http::handlers::auth_handler::{login_handler, logout_handler, me_handler, oauth_callback_handler, refresh_handler};

pub fn auth_routes() -> Router<AppState> {
        Router::new()
            .route("/me",       get(me_handler))
            .route("/login",    get(login_handler))
            .route("/logout",   post(logout_handler))
            .route("/refresh",  get(refresh_handler))
            .route("/callback", get(oauth_callback_handler))
            .route_layer(SetResponseHeaderLayer::if_not_present(
                CACHE_CONTROL,
                HeaderValue::from_static("no-store"),
            ))
            .route_layer(SetResponseHeaderLayer::if_not_present(
                PRAGMA,
                HeaderValue::from_static("no-cache"),
            ))
}