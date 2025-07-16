use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum_extra::TypedHeader;
use axum_extra::headers::{authorization::Bearer, Authorization};
use crate::security::keycloak::claims::KeycloakClaims;
use crate::security::keycloak::validator::validate_token_and_extract_claims;

pub struct Claims(pub KeycloakClaims);

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            let TypedHeader(Authorization(bearer)) =
                TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, _state)
                    .await
                    .map_err(|_| (StatusCode::UNAUTHORIZED, "Missing Authorization".into()))?;

            let token = bearer.token();

            match validate_token_and_extract_claims(token).await {
                Ok(claims) => {
                    tracing::info!(
                        "JWT verified for cache with roles: {:?}",
                        claims.realm_access.roles
                    );
                    Ok(Claims(claims))
                }
                Err(err) => {
                    tracing::warn!("JWT validation failed: {:?}", err);
                    Err((StatusCode::UNAUTHORIZED, format!("JWT validation failed: {}", err)))
                }
            }
        }
    }
}
