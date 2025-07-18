use std::env;
use axum::{Json};
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use reqwest::StatusCode;
use crate::app_state::AppState;
use crate::dto::auth::auth_callback_request::AuthCallbackRequest;

pub async fn oauth_callback_handler(
    Query(query): Query<AuthCallbackRequest>,
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    state
        .auth_service
        .exchange_code_for_token(query.code).await
        .map(|token| {
            let cookie = Cookie::build(("access_token", token.access_token))
                .http_only(true)
                .path("/")
                .build();

            let refresh_cookie = Cookie::build(("refresh_token", token.refresh_token.unwrap_or_default()))
                .http_only(true)
                .path("/")
                .build();

            let jar = jar.add(cookie).add(refresh_cookie);

            let frontend_url = env::var("FRONTEND_URL").unwrap_or_else(|_| "/".to_string());
            (jar, Redirect::temporary(&frontend_url))
        })
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Auth failed: {e}")))
}

pub async fn refresh_handler(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if let Some(refresh_cookie) = jar.get("refresh_token") {
        let refresh_token = refresh_cookie.value().to_string();

        match state.auth_service.refresh_access_token(&refresh_token).await {
            Ok(token) => {
                let access_cookie = Cookie::build(("access_token", token.access_token.clone()))
                    .http_only(true)
                    .path("/")
                    .build();

                let refresh_cookie = Cookie::build(("refresh_token", token.refresh_token.clone().unwrap_or_default()))
                    .http_only(true)
                    .path("/")
                    .build();

                let jar = jar.add(access_cookie).add(refresh_cookie);

                Ok((jar, Json(token)))
            }
            Err(e) => Err((StatusCode::UNAUTHORIZED, format!("Refresh failed: {e}"))),
        }
    } else {
        Err((StatusCode::UNAUTHORIZED, "No refresh token found".to_string()))
    }
}

pub async fn login_handler(State(state): State<AppState>) -> Redirect {
    Redirect::to(&state.auth_service.build_authorize_url())
}

pub async fn logout_handler(mut jar: CookieJar) -> impl IntoResponse {
    jar = jar.remove(
        Cookie::build("access_token")
            .path("/")
            .build()
    );
    jar = jar.remove(
        Cookie::build("refresh_token")
            .path("/")
            .build()
    );
    (jar, Redirect::temporary("/"))
}