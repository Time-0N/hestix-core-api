use axum_extra::extract::cookie::{Cookie, SameSite};
use time::{Duration, OffsetDateTime};

/// Check if the application is running in production mode
fn is_production() -> bool {
    std::env::var("ENVIRONMENT")
        .unwrap_or_else(|_| "development".to_string())
        .to_lowercase() == "production"
}

pub fn access_cookie(token: String) -> Cookie<'static> {
    Cookie::build(("access_token", token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(is_production()) // Only secure in production
        .expires(OffsetDateTime::now_utc() + Duration::hours(1))
        .build()
}

pub fn refresh_cookie(token: String) -> Cookie<'static> {
    Cookie::build(("refresh_token", token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(is_production()) // Only secure in production
        .expires(OffsetDateTime::now_utc() + Duration::days(7)) // Reduced from 30 days for better security
        .build()
}

pub fn oauth_state_cookie(state: String) -> Cookie<'static> {
    Cookie::build(("oauth_state", state))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(is_production()) // Only secure in production
        .expires(OffsetDateTime::now_utc() + Duration::minutes(10))
        .build()
}

pub fn pkce_verifier_cookie(verifier: String) -> Cookie<'static> {
    Cookie::build(("pkce_verifier", verifier))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(is_production()) // Only secure in production
        .expires(OffsetDateTime::now_utc() + Duration::minutes(10))
        .build()
}

pub fn remove_cookie(name: &str) -> Cookie<'static> {
    Cookie::build((name.to_string(), ""))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(is_production()) // Only secure in production
        .expires(OffsetDateTime::now_utc() - Duration::days(1))
        .build()
}