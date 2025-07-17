use axum::http::{header, HeaderValue, Method};
use tower_http::cors::CorsLayer;
use tower_http::set_header::SetResponseHeaderLayer;

pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin("http://localhost:4200".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
        .allow_credentials(true)
}

pub fn security_headers_layer() -> SetResponseHeaderLayer<HeaderValue> {
    SetResponseHeaderLayer::overriding(
        header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=63072000; includeSubDomains; preload"),
    )
}
