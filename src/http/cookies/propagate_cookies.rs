use axum::{
    body::Body,
    http::Request,
    middleware::Next,
    response::{Response, IntoResponse},
};
use axum_extra::extract::cookie::CookieJar;

pub async fn propagate_cookies(
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let jar = req
        .extensions_mut()
        .remove::<CookieJar>()
        .unwrap_or_else(CookieJar::new);

    let response = next.run(req).await;

    (jar, response).into_response()
}
