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
use crate::oidc::OidcClaims;

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
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        // Get a concrete AppState for provider access
        let app = AppState::from_ref(state);

        async move {
            // 1) Authorization: Bearer <token>
            if let Ok(TypedHeader(Authorization(bearer))) =
                TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state).await
            {
                let claims = app
                    .auth_service
                    .validate(bearer.token())
                    .await
                    .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()).into_response())?;
                return Ok(Self(claims));
            }

            // 2) Cookie fallback
            let jar = CookieJar::from_request_parts(parts, state)
                .await
                .map_err(|_| (StatusCode::UNAUTHORIZED, "Missing CookieJar").into_response())?;

            if let Some(c) = jar.get("access_token") {
                let claims = app
                    .auth_service
                    .validate(c.value())
                    .await
                    .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()).into_response())?;
                return Ok(Self(claims));
            }

            // 3) Optional refresh flow if you keep refresh cookies
            if let Some(refresh) = jar.get("refresh_token") {
                let pair = app
                    .auth_service
                    .refresh_access_token(refresh.value())
                    .await
                    .map_err(|e| {
                        (StatusCode::UNAUTHORIZED, format!("Token refresh failed: {e}"))
                            .into_response()
                    })?;

                // Validate new token
                let claims = app
                    .auth_service
                    .validate(&pair.access_token)
                    .await
                    .map_err(|e| {
                        (StatusCode::UNAUTHORIZED, format!("New token invalid: {e}"))
                            .into_response()
                    })?;

                // Reset cookies on the response via extensions
                let access_cookie = Cookie::build(("access_token", pair.access_token.clone()))
                    .http_only(true)
                    .path("/")
                    .build();
                let refresh_cookie = Cookie::build((
                    "refresh_token",
                    pair.refresh_token.clone().unwrap_or_default(),
                ))
                    .http_only(true)
                    .path("/")
                    .build();

                let new_jar = jar.clone().add(access_cookie).add(refresh_cookie);
                parts.extensions.insert(new_jar);

                return Ok(Self(claims));
            }

            Err((StatusCode::UNAUTHORIZED, "Missing Authorization or access_token").into_response())
        }
    }
}
