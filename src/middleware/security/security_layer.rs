use std::env;
use axum::http::{header, HeaderValue, Method};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, warn, error};
use crate::app_state::AppState;

pub fn cors_layer() -> CorsLayer {
    let key_single = "CORS_ALLOWED_ORIGIN";
    let key_multi  = "CORS_ALLOWED_ORIGINS";

    let raw = env::var(key_multi)
        .or_else(|_| env::var(key_single))
        .ok();

    match raw {
        Some(list) => {
            // split + parse; bail if any parse fails
            let mut origins = Vec::<HeaderValue>::new();
            for s in list.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
                match s.parse::<HeaderValue>() {
                    Ok(hv) => origins.push(hv),
                    Err(e) => {
                        error!(origin = %s, error = %e, "Invalid CORS origin in env");
                        // refuse to run with a partly-invalid list to avoid surprises
                        return CorsLayer::new();
                    }
                }
            }

            if origins.is_empty() {
                warn!("CORS_* set but no valid origins parsed; all cross-origin requests will be denied.");
                return CorsLayer::new();
            }

            info!(origins=%list, "CORS allowed for: ");

            CorsLayer::new()
                .allow_origin(origins)
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
                .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
                .allow_credentials(true)
        }
        None => {
            warn!("CORS_ALLOWED_ORIGINS / CORS_ALLOWED_ORIGIN not set. All cross-origin requests will be denied.");
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
        SetResponseHeaderLayer::if_not_present(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("default-src 'self'; frame-ancestors 'none'"),
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