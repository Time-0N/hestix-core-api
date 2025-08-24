use axum::http::StatusCode;
use std::fmt::Debug;

pub fn auth_fail<E: Debug>(msg: &'static str) -> impl FnOnce(E) -> (StatusCode, String) {
    move |e| {
        tracing::warn!(error=?e, "auth error: {msg}");
        (StatusCode::UNAUTHORIZED, "authentication failed".to_string())
    }
}

pub fn server_fail<E: Debug>(msg: &'static str) -> impl FnOnce(E) -> (StatusCode, String) {
    move |e| {
        tracing::error!(error=?e, "server error: {msg}");
        (StatusCode::INTERNAL_SERVER_ERROR, "internal server error".to_string())
    }
}
