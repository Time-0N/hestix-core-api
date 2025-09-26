use axum::http::{header, HeaderValue};
use tower_http::set_header::SetResponseHeaderLayer;

/// Build security headers for HTTP responses
pub fn build_security_headers() -> Vec<SetResponseHeaderLayer<HeaderValue>> {
    vec![
        // HSTS (HTTP Strict Transport Security)
        SetResponseHeaderLayer::overriding(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=63072000; includeSubDomains; preload"),
        ),

        // Prevent MIME type sniffing
        SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        ),

        // Prevent framing (clickjacking protection)
        SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY"),
        ),

        // Referrer policy
        SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("referrer-policy"),
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ),

        // Permissions policy (feature restrictions)
        SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("permissions-policy"),
            HeaderValue::from_static("geolocation=(), camera=()"),
        ),

        // Content Security Policy (only if not already set)
        SetResponseHeaderLayer::if_not_present(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("default-src 'self'; frame-ancestors 'none'"),
        ),
    ]
}