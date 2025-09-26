use axum::{
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use axum_extra::{
    extract::cookie::{Cookie, CookieJar},
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

use crate::app_state::AppState;
use crate::infrastructure::oidc::OidcClaims;

pub enum TokenSource {
    Bearer(String),
    Cookie(String),
}

/// Extract token from Authorization header or access_token cookie
pub async fn extract_token_from_request<S>(
    parts: &mut Parts,
    state: &S,
) -> Result<TokenSource, Response>
where
    S: Send + Sync + 'static,
    AppState: FromRef<S>,
{
    // 1) Try Authorization: Bearer <token>
    if let Ok(TypedHeader(Authorization(bearer))) =
        TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state).await
    {
        return Ok(TokenSource::Bearer(bearer.token().to_string()));
    }

    // 2) Try Cookie fallback
    let jar = CookieJar::from_request_parts(parts, state)
        .await
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Missing CookieJar").into_response())?;

    if let Some(c) = jar.get("access_token") {
        return Ok(TokenSource::Cookie(c.value().to_string()));
    }

    Err((StatusCode::UNAUTHORIZED, "Missing Authorization or access_token").into_response())
}

/// Validate a token and return claims
pub async fn validate_token(
    app_state: &AppState,
    token: &str,
) -> Result<OidcClaims, Response> {
    let claims = app_state
        .auth_service
        .validate(token)
        .await
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()).into_response())?;

    // Additional expiration check (defense in depth)
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if claims.exp <= now {
        return Err((StatusCode::UNAUTHORIZED, "Token expired").into_response());
    }

    // Check if token is too old (issued more than reasonable time ago)
    if now.saturating_sub(claims.iat) > 86400 { // 24 hours
        return Err((StatusCode::UNAUTHORIZED, "Token too old").into_response());
    }

    Ok(claims)
}

/// Attempt to refresh tokens using refresh_token cookie
pub async fn attempt_token_refresh<S>(
    parts: &mut Parts,
    state: &S,
) -> Result<(OidcClaims, CookieJar), Response>
where
    S: Send + Sync + 'static,
    AppState: FromRef<S>,
{
    let app = AppState::from_ref(state);

    let jar = CookieJar::from_request_parts(parts, state)
        .await
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Missing CookieJar").into_response())?;

    let refresh_token = jar.get("refresh_token")
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Missing refresh token").into_response())?;

    let token_pair = app
        .auth_service
        .refresh_access_token(refresh_token.value())
        .await
        .map_err(|e| {
            (StatusCode::UNAUTHORIZED, format!("Token refresh failed: {e}"))
                .into_response()
        })?;

    // Validate new token
    let claims = validate_token(&app, &token_pair.access_token).await?;

    /// Check if the application is running in production mode
    fn is_production() -> bool {
        std::env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string())
            .to_lowercase() == "production"
    }

    // Build new cookies
    let access_cookie = Cookie::build(("access_token", token_pair.access_token.clone()))
        .http_only(true)
        .path("/")
        .secure(is_production()) // Only secure in production
        .build();

    let refresh_cookie = Cookie::build((
        "refresh_token",
        token_pair.refresh_token.clone().unwrap_or_default(),
    ))
        .http_only(true)
        .path("/")
        .secure(is_production()) // Only secure in production
        .build();

    let new_jar = jar.clone().add(access_cookie).add(refresh_cookie);

    Ok((claims, new_jar))
}