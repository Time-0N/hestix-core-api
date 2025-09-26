use std::env;
use axum::http::{header, HeaderValue, Method};
use tower_http::cors::CorsLayer;
use tracing::{info, warn, error};

pub fn build_cors_layer() -> CorsLayer {
    let key_single = "CORS_ALLOWED_ORIGIN";
    let key_multi = "CORS_ALLOWED_ORIGINS";

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