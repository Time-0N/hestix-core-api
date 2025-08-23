use axum::{Json};
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use http::StatusCode;
use serde::Deserialize;
use serde_json::Value;
use base64::{engine::general_purpose, Engine as _};
use sha2::{Digest, Sha256};
use ring::rand::{SecureRandom, SystemRandom};
use time::Duration;

use crate::app_state::AppState;
use crate::middleware::security::extractor::Claims;

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
    println!("=== LOGIN HANDLER CALLED ===");
    println!("Existing cookies: {:?}", jar.iter().map(|c| (c.name(), c.value())).collect::<Vec<_>>());

    // ALWAYS clear any existing auth-related cookies first to prevent loops
    let jar = jar
        .remove(Cookie::build("pkce_verifier").path("/").build())
        .remove(Cookie::build("oauth_state").path("/").build())
        .remove(Cookie::build("access_token").path("/").build())
        .remove(Cookie::build("refresh_token").path("/").build());

    // Always generate fresh PKCE pair
    let (verifier, challenge) = generate_pkce_pair();
    let state_str = random_b64url(32);

    println!("Generated NEW state: {}", state_str);
    println!("Generated NEW verifier: {}", verifier);

    // Create cookies with fresh values
    let jar = jar
        .add(
            Cookie::build(("pkce_verifier", verifier.clone()))
                .http_only(true)
                .secure(false)
                .same_site(SameSite::Lax)
                .path("/")
                .max_age(Duration::minutes(15))
                .build(),
        )
        .add(
            Cookie::build(("oauth_state", state_str.clone()))
                .http_only(true)
                .secure(false)
                .same_site(SameSite::Lax)
                .path("/")
                .max_age(Duration::minutes(15))
                .build(),
        );

    let url = state.auth_service
        .build_authorize_url(Some(&challenge), Some(state_str.clone()))
        .await;

    println!("Redirecting to: {}", url);
    println!("Set NEW cookies: pkce_verifier, oauth_state");

    (jar, Redirect::to(&url))
}

pub async fn oauth_callback_handler(
    Query(query): Query<AuthCallbackRequest>,
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    println!("=== CALLBACK START ===");
    println!("Code: {}", query.code);
    println!("State: {:?}", query.state);
    println!("Available cookies: {:?}", jar.iter().map(|c| (c.name(), c.value())).collect::<Vec<_>>());

    // Get state from query parameter
    let received_state = query
        .state
        .as_deref()
        .ok_or_else(|| {
            println!("ERROR: Missing state parameter");
            (StatusCode::BAD_REQUEST, "missing state parameter".to_string())
        })?;

    // Get stored state from cookie
    let stored_state = jar
        .get("oauth_state")
        .ok_or_else(|| {
            println!("ERROR: Missing oauth_state cookie");
            (StatusCode::BAD_REQUEST, "missing oauth_state cookie".to_string())
        })?
        .value();

    println!("Received state: {}", received_state);
    println!("Stored state: {}", stored_state);

    // Verify state matches (CSRF protection)
    if received_state != stored_state {
        println!("ERROR: State mismatch! Received: {}, Stored: {}", received_state, stored_state);
        return Err((StatusCode::BAD_REQUEST, "state mismatch".to_string()));
    }

    // Get PKCE verifier from cookie
    let verifier = jar
        .get("pkce_verifier")
        .ok_or_else(|| {
            println!("ERROR: Missing pkce_verifier cookie");
            (StatusCode::BAD_REQUEST, "missing pkce_verifier cookie".to_string())
        })?
        .value()
        .to_string();

    println!("Using verifier: {}", verifier);

    // Exchange code for tokens
    println!("Attempting token exchange...");
    let token = state
        .auth_service
        .exchange_code_for_token(query.code, Some(verifier))
        .await
        .map_err(|e| {
            println!("ERROR: Token exchange failed: {}", e);
            (StatusCode::UNAUTHORIZED, format!("token exchange failed: {}", e))
        })?;

    println!("Token exchange successful!");

    // Clear temporary auth cookies and set real auth cookies
    let mut jar = jar
        .remove(Cookie::build("pkce_verifier").path("/").build())
        .remove(Cookie::build("oauth_state").path("/").build());

    jar = jar.add(
        Cookie::build(("access_token", token.access_token.clone()))
            .http_only(true)
            .secure(false) // Set to true in production with HTTPS
            .same_site(SameSite::Lax)
            .path("/")
            .max_age(Duration::hours(1))
            .build(),
    );

    if let Some(rt) = token.refresh_token.clone() {
        jar = jar.add(
            Cookie::build(("refresh_token", rt))
                .http_only(true)
                .secure(false) // Set to true in production with HTTPS
                .same_site(SameSite::Lax)
                .path("/")
                .max_age(Duration::days(7))
                .build(),
        );
    }

    // Get frontend URL and redirect
    let target = std::env::var("FRONTEND_URL")
        .unwrap_or_else(|_| "http://localhost:5173".to_string());

    println!("Redirecting to frontend: {}", target);
    println!("=== CALLBACK END ===");
    Ok((jar, Redirect::to(&target)))
}

pub async fn refresh_handler(
    State(state): State<AppState>,
    jar: CookieJar
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let Some(refresh_cookie) = jar.get("refresh_token") else {
        return Err((StatusCode::UNAUTHORIZED, "no refresh token".into()));
    };

    let token = state.auth_service
        .refresh_access_token(refresh_cookie.value())
        .await
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    let mut jar = jar;
    jar = jar.add(
        Cookie::build(("access_token", token.access_token.clone()))
            .http_only(true)
            .secure(false) // Set to true in production with HTTPS
            .same_site(SameSite::Lax)
            .path("/")
            .max_age(Duration::hours(1))
            .build()
    );

    if let Some(rt) = token.refresh_token.clone() {
        jar = jar.add(
            Cookie::build(("refresh_token", rt))
                .http_only(true)
                .secure(false) // Set to true in production with HTTPS
                .same_site(SameSite::Lax)
                .path("/")
                .max_age(Duration::days(7))
                .build()
        );
    }

    Ok((jar, Json(serde_json::json!({"status": "refreshed"}))))
}

pub async fn logout_handler(jar: CookieJar) -> impl IntoResponse {
    let jar = jar
        .remove(Cookie::build("access_token").path("/").build())
        .remove(Cookie::build("refresh_token").path("/").build())
        .remove(Cookie::build("pkce_verifier").path("/").build())
        .remove(Cookie::build("oauth_state").path("/").build());

    let target = std::env::var("FRONTEND_URL")
        .unwrap_or_else(|_| "http://localhost:5173".to_string());
    (jar, Redirect::to(&target))
}

pub async fn me_handler(Claims(claims): Claims) -> Json<Value> {
    Json(serde_json::json!(claims))
}