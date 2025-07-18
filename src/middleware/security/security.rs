use std::env;
use axum::http::{header, HeaderValue, Method};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;
use crate::app_state::AppState;

pub fn cors_layer() -> CorsLayer {
    match env::var("CORS_ALLOWED_ORIGIN") {
        Ok(origin) => {
            println!("CORS allowed for origin: {origin}");
            CorsLayer::new()
                .allow_origin(origin.parse::<HeaderValue>().expect("Invalid CORS origin"))
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
                .allow_credentials(true)
        }
        Err(_) => {
            eprintln!("WARNING: CORS_ALLOWED_ORIGIN is not set. All cross-origin requests will be denied.");
            CorsLayer::new()
        }
    }
}

pub fn security_header_layers() -> Vec<SetResponseHeaderLayer<HeaderValue>> {
    vec![
        SetResponseHeaderLayer::overriding(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=63072000; includeSubDomains; preload"),
        ),
        SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        ),
        SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY"),
        ),
        SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("referrer-policy"),
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ),
        SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("permissions-policy"),
            HeaderValue::from_static("geolocation=(), camera=()"),
        ),
    ]
}

pub fn apply_security_layers(router: Router<AppState>) -> Router<AppState> {
    let mut app = router.layer(cors_layer());

    for header_layer in security_header_layers() {
        app = app.layer(header_layer);
    }

    app.layer(TraceLayer::new_for_http())
}