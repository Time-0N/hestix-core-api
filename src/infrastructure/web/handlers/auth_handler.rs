use axum::{Json};
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect};
use axum_extra::extract::cookie::{CookieJar};
use http::StatusCode;
use serde::Deserialize;
use serde_json::Value;
use base64::{engine::general_purpose, Engine as _};
use sha2::{Digest, Sha256};
use ring::rand::{SecureRandom, SystemRandom};
use tracing::{debug, info};

use crate::app_state::AppState;
use crate::infrastructure::web::cookies::cookie_helper::{access_cookie, oauth_state_cookie, pkce_verifier_cookie, refresh_cookie, remove_cookie};
use crate::shared::middleware::security::extractor::Claims;
use crate::infrastructure::web::errors::auth_fail;

#[derive(Deserialize)]
pub struct AuthCallbackRequest {
    pub code: String,
    pub state: Option<String>
}

fn generate_pkce_pair() -> (String, String) {
    let rng = SystemRandom::new();
    let mut verifier_bytes = [0u8; 32];
    rng.fill(&mut verifier_bytes).expect("OS RNG");

    let code_verifier = general_purpose::URL_SAFE_NO_PAD.encode(verifier_bytes);
    let digest = Sha256::digest(code_verifier.as_bytes());
    let code_challenge = general_purpose::URL_SAFE_NO_PAD.encode(digest);

    (code_verifier, code_challenge)
}

fn random_b64url(len: usize) -> String {
    let rng = SystemRandom::new();
    let mut bytes = vec![0u8; len];
    rng.fill(&mut bytes).expect("OS RNG");
    general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

pub async fn login_handler(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    // Clear any existing auth cookies
    let jar = jar
        .remove(remove_cookie("pkce_verifier"))
        .remove(remove_cookie("oauth_state"))
        .remove(remove_cookie("access_token"))
        .remove(remove_cookie("refresh_token"));

    let (verifier, challenge) = generate_pkce_pair();
    let state_str = random_b64url(32);

    // Mask sensitive values if you *must* log
    debug!(state_len = state_str.len(), "generated oauth state");
    debug!("set pkce/state cookies");

    let jar = jar
        .add(pkce_verifier_cookie(verifier.clone()))
        .add(oauth_state_cookie(state_str.clone()));

    let url = state.auth_service
        .build_authorize_url(Some(&challenge), Some(state_str.clone()))
        .await;

    debug!(%url, "redirecting to provider");
    (jar, Redirect::to(&url))
}

pub async fn oauth_callback_handler(
    Query(query): Query<AuthCallbackRequest>,
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<impl IntoResponse, (StatusCode, String)> {

    debug!(has_state=%query.state.is_some(), "callback received");

    let received_state = query.state.as_deref()
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "missing state parameter".to_string()))?;

    let stored_state = jar.get("oauth_state")
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "missing oauth_state cookie".to_string()))?
        .value();

    // Avoid logging values; if you must:
    debug!(recv_len = received_state.len(), stored_len = stored_state.len(), "verifying state");
    if received_state != stored_state {
        return Err((StatusCode::BAD_REQUEST, "state mismatch".to_string()));
    }

    let verifier = jar.get("pkce_verifier")
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "missing pkce_verifier cookie".to_string()))?
        .value()
        .to_string();

    debug!("exchanging code for token");
    let token = state
        .auth_service
        .exchange_code_for_token(query.code, Some(verifier))
        .await
        .map_err(auth_fail("token exchange failed"))?;
    info!("token exchange successful");

    // Clear temp cookies, set real ones via helpers
    let mut jar = jar
        .remove(remove_cookie("pkce_verifier"))
        .remove(remove_cookie("oauth_state"));

    jar = jar.add(access_cookie(token.access_token.clone()));
    if let Some(rt) = token.refresh_token.clone() {
        jar = jar.add(refresh_cookie(rt));
    }

    let target = std::env::var("FRONTEND_URL")
        .unwrap_or_else(|_| "http://localhost:5173".to_string());

    Ok((jar, Redirect::to(&target)))
}

pub async fn refresh_handler(
    State(state): State<AppState>,
    jar: CookieJar
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let Some(refresh_cookie_value) = jar.get("refresh_token") else {
        return Err((StatusCode::UNAUTHORIZED, "no refresh token".into()));
    };

    let token = state.auth_service
        .refresh_access_token(refresh_cookie_value.value())
        .await
        .map_err(auth_fail("refresh failed"))?;

    let mut jar = jar;
    jar = jar.add(access_cookie(token.access_token.clone()));
    if let Some(rt) = token.refresh_token.clone() {
        jar = jar.add(refresh_cookie(rt));
    }

    Ok((jar, Json(serde_json::json!({"status": "refreshed"}))))
}

pub async fn logout_handler(jar: CookieJar) -> impl IntoResponse {
    let jar = jar
        .remove(remove_cookie("access_token"))
        .remove(remove_cookie("refresh_token"))
        .remove(remove_cookie("pkce_verifier"))
        .remove(remove_cookie("oauth_state"));

    let target = std::env::var("FRONTEND_URL")
        .unwrap_or_else(|_| "http://localhost:5173".to_string());
    (jar, Redirect::to(&target))
}

pub async fn me_handler(Claims(claims): Claims) -> Json<Value> {
    Json(serde_json::json!(claims))
}