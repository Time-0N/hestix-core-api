use once_cell::sync::Lazy;
use axum_extra::extract::cookie::{Cookie, SameSite};
use time::{Duration};

static COOKIE_SECURE: Lazy<bool> = Lazy::new(|| {
    std::env::var("COOKIE_SECURE")
        .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE"))
        .unwrap_or(false)
});

#[inline]
fn cookie_secure() -> bool { *COOKIE_SECURE }

const ACCESS_MAX_AGE:  Duration = Duration::hours(1);
const REFRESH_MAX_AGE: Duration = Duration::days(7);
const TEMP_MAX_AGE:    Duration = Duration::minutes(15);

fn base_cookie(name: &str, value: String) -> Cookie<'static> {
    Cookie::build((name.to_owned(), value))
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(cookie_secure())
        .path("/")
        .build()
}

// --- Auth cookies ---
pub fn access_cookie(token: String) -> Cookie<'static> {
    let mut c = base_cookie("access_token", token);
    c.set_max_age(ACCESS_MAX_AGE);
    c
}

pub fn refresh_cookie(token: String) -> Cookie<'static> {
    let mut c = base_cookie("refresh_token", token);
    c.set_max_age(REFRESH_MAX_AGE);
    c
}

// --- Temp cookies for PKCE/state ---
pub fn pkce_verifier_cookie(verifier: String) -> Cookie<'static> {
    let mut c = base_cookie("pkce_verifier", verifier);
    c.set_max_age(TEMP_MAX_AGE);
    c
}

pub fn oauth_state_cookie(state: String) -> Cookie<'static> {
    let mut c = base_cookie("oauth_state", state);
    c.set_max_age(TEMP_MAX_AGE);
    c
}

// --- Removal helpers ---
pub fn remove_cookie(name: &str) -> Cookie<'static> {
    Cookie::build((name.to_owned(), String::new()))
        .path("/")
        .build()
}

pub fn clear_auth_cookies(jar: axum_extra::extract::cookie::CookieJar)
                          -> axum_extra::extract::cookie::CookieJar
{
    jar
        .remove(remove_cookie("access_token"))
        .remove(remove_cookie("refresh_token"))
        .remove(remove_cookie("pkce_verifier"))
        .remove(remove_cookie("oauth_state"))
}
