use axum::{
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use std::future::Future;

use crate::app_state::AppState;
use crate::infrastructure::oidc::OidcClaims;

use super::tokens::{extract_token_from_request, validate_token, attempt_token_refresh, TokenSource};

pub struct Claims(pub OidcClaims);

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync + 'static,
    AppState: FromRef<S>,
{
    type Rejection = Response;

    fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        let app = AppState::from_ref(state);

        async move {
            // Try to extract and validate existing token
            match extract_token_from_request(parts, state).await {
                Ok(token_source) => {
                    let (token, is_cookie) = match &token_source {
                        TokenSource::Bearer(t) => (t.clone(), false),
                        TokenSource::Cookie(t) => (t.clone(), true),
                    };

                    match validate_token(&app, &token).await {
                        Ok(claims) => return Ok(Self(claims)),
                        Err(_) => {
                            // Token validation failed, try refresh if it was a cookie token
                            if is_cookie {
                                // Fall through to refresh attempt
                            } else {
                                return Err((StatusCode::UNAUTHORIZED, "Invalid Bearer token").into_response());
                            }
                        }
                    }
                }
                Err(_) => {
                    // No token found, try refresh
                }
            }

            // Attempt token refresh
            match attempt_token_refresh(parts, state).await {
                Ok((claims, new_jar)) => {
                    // Store the new jar in request extensions for later propagation
                    parts.extensions.insert(new_jar);
                    Ok(Self(claims))
                }
                Err(e) => Err(e),
            }
        }
    }
}