use axum::{
    extract::{FromRequestParts},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use axum_extra::{
    extract::cookie::{Cookie, CookieJar},
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use crate::app_state::AppState;
use crate::dto::auth::claims::KeycloakClaims;
use crate::middleware::security::keycloak::validator::validate_token_and_extract_claims;

pub struct Claims(pub KeycloakClaims);

impl FromRequestParts<AppState> for Claims {
    type Rejection = Response;


    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let svc = state.auth_service.clone();

        let jar = CookieJar::from_request_parts(parts, &())
            .await
            .map_err(|_| {
                (StatusCode::UNAUTHORIZED, "Missing CookieJar").into_response()
            })?;

        let token = if let Ok(TypedHeader(Authorization(bearer))) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, &()).await
        {
            bearer.token().to_owned()
        } else if let Some(access_cookie) = jar.get("access_token") {
            access_cookie.value().to_owned()
        } else {
            return Err((
                StatusCode::UNAUTHORIZED,
                "Missing Authorization header or access_token cookie",
            )
                .into_response());
        };

        match validate_token_and_extract_claims(&token).await {
            Ok(claims) => Ok(Self(claims)),
            Err(_) => {
                let Some(refresh_cookie) = jar.get("refresh_token") else {
                    return Err((
                        StatusCode::UNAUTHORIZED,
                        "Access token expired and no refresh token found",
                    )
                        .into_response());
                };

                let new_pair = svc
                    .refresh_access_token(refresh_cookie.value())
                    .await
                    .map_err(|e| {
                        (
                            StatusCode::UNAUTHORIZED,
                            format!("Token refresh failed: {e}"),
                        )
                            .into_response()
                    })?;

                let claims = validate_token_and_extract_claims(&new_pair.access_token)
                    .await
                    .map_err(|e| {
                        (
                            StatusCode::UNAUTHORIZED,
                            format!("New token invalid: {e}"),
                        )
                            .into_response()
                    })?;

                let access_cookie = Cookie::build(("access_token", new_pair.access_token.clone()))
                    .http_only(true)
                    .path("/")
                    .build();
                let refresh_cookie = Cookie::build((
                    "refresh_token",
                    new_pair.refresh_token.unwrap_or_default(),
                ))
                    .http_only(true)
                    .path("/")
                    .build();

                let new_jar = jar.clone().add(access_cookie).add(refresh_cookie);
                parts.extensions.insert(new_jar);

                Ok(Self(claims))
            }
        }
    }
}