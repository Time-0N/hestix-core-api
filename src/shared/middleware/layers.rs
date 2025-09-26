use axum::Router;
use tower_http::trace::TraceLayer;

use crate::app_state::AppState;

use super::{
    cors::build_cors_layer,
    headers::build_security_headers,
};

/// Apply all security-related middleware layers to the router
pub fn apply_security_layers(router: Router<AppState>) -> Router<AppState> {
    let mut app = router
        .layer(build_cors_layer())
        .layer(TraceLayer::new_for_http());

    // Apply all security headers
    for header_layer in build_security_headers() {
        app = app.layer(header_layer);
    }

    app
}