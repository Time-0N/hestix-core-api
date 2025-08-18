use axum::{Json};
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use http::StatusCode;
use serde::Deserialize;
use serde_json::Value;
use base64::{engine::general_purpose, Engine as _};
use sha2::{Digest, Sha256};
use ring::rand::{SecureRandom, SystemRandom};

use crate::app_state::AppState;
use crate::middleware::security::extractor::Claims;

#[derive(Deserialize)]
pub struct AuthCallbackRequest { pub code: String, pub state: Option<String> }


fn generate_pkce_pair() -> (String, String) {
    // 32 bytes -> 43 chars base64url(no pad), valid PKCE length
    let rng = SystemRandom::new();
    let mut verifier_bytes = [0u8; 32];
    rng.fill(&mut verifier_bytes).expect("OS RNG");

    let code_verifier = general_purpose::URL_SAFE_NO_PAD.encode(verifier_bytes);
    let digest = Sha256::digest(code_verifier.as_bytes());
    let code_challenge = general_purpose::URL_SAFE_NO_PAD.encode(digest);

    (code_verifier, code_challenge)
}

pub async fn login_handler(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    let (verifier, challenge) = generate_pkce_pair();
    let jar = jar.add(
        Cookie::build(("pkce_verifier", verifier))
            .http_only(true)
            .path("/")
            .build()
    );
    let url = state.auth_service.build_authorize_url(Some(&challenge), None).await;
    (jar, Redirect::to(&url))
}

pub async fn oauth_callback_handler(
    Query(query): Query<AuthCallbackRequest>,
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let Some(verifier_cookie) = jar.get("pkce_verifier") else {
        return Err((StatusCode::BAD_REQUEST, "missing pkce verifier".into()));
    };
    let verifier = verifier_cookie.value().to_string();

    let token = state
        .auth_service
        .exchange_code_for_token(query.code, Some(verifier))
        .await
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    let mut jar = jar;
    jar = jar.add(
        Cookie::build(("access_token", token.access_token.clone()))
            .http_only(true)
            .path("/")
            .build()
    );
    if let Some(rt) = token.refresh_token.clone() {
        jar = jar.add(
            Cookie::build(("refresh_token", rt))
                .http_only(true)
                .path("/")
                .build()
        );
    }
    jar = jar.remove(Cookie::build("pkce_verifier").path("/").build());

    let target = std::env::var("FRONTEND_URL")
        .unwrap_or_else(|_| "http://localhost:3000/v1".to_string());

    Ok((jar, Redirect::to(&target)))
}

pub async fn refresh_handler(State(state): State<AppState>, jar: CookieJar) -> Result<impl IntoResponse, (StatusCode, String)> {
    let Some(refresh_cookie) = jar.get("refresh_token") else {
        return Err((StatusCode::UNAUTHORIZED, "no refresh token".into()));
    };
    let token = state.auth_service.refresh_access_token(refresh_cookie.value()).await
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    let mut jar = jar;
    jar = jar.add(
        Cookie::build(("access_token", token.access_token.clone()))
            .http_only(true)
            .path("/")
            .build()
    );
    if let Some(rt) = token.refresh_token.clone() {
        jar = jar.add(
            Cookie::build(("refresh_token", rt))
                .http_only(true)
                .path("/")
                .build()
        );
    }
    Ok((jar, Redirect::temporary("/")))
}

pub async fn logout_handler(jar: CookieJar) -> impl IntoResponse {
    let jar = jar
        .remove(Cookie::build("access_token").path("/").build())
        .remove(Cookie::build("refresh_token").path("/").build());

    let target = std::env::var("FRONTEND_URL")
        .unwrap_or_else(|_| "http://localhost:4200/".to_string());

    (jar, Redirect::to(&target))
}

pub async fn me_handler(Claims(claims): Claims) -> Json<Value> {
    Json(serde_json::json!(claims))
}
