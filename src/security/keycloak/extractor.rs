use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum_extra::TypedHeader;
use axum_extra::headers::{authorization::Bearer, Authorization};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use crate::security::keycloak::claims::KeycloakClaims;
use crate::security::keycloak::jwk::{get_cached_jwks};

pub struct Claims(pub KeycloakClaims);

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            let TypedHeader(Authorization(bearer)) =
                TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                    .await
                    .map_err(|_| (StatusCode::UNAUTHORIZED, "Missing Authorization".into()))?;
            let token = bearer.token();

            let header = decode_header(token)
                .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid JWT header: {}", e)))?;
            let kid = header.kid
                .ok_or((StatusCode::BAD_REQUEST, "No `kid` in JWT".into()))?;

            let jwks = get_cached_jwks()
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

            let jwk = jwks
                .keys
                .iter()
                .find(|j| j.common.key_id.as_deref() == Some(&kid))
                .ok_or((StatusCode::UNAUTHORIZED, "Unknown JWK `kid`".into()))?;

            let alg = match &jwk.algorithm {
                jsonwebtoken::jwk::AlgorithmParameters::RSA(rsa) => rsa,
                _ => {
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Unsupported key type".into(),
                    ))
                }
            };

            let allowed_audiences = std::env::var("KEYCLOAK_ALLOWED_AUDIENCES")
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Missing env var KEYCLOAK_ALLOWED_AUDIENCES".into()))?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect::<Vec<String>>();

            let base_url = std::env::var("KEYCLOAK_BASE_URL")
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Missing env var KEYCLOAK_BASE_URL".into()))?;

            let realm = std::env::var("KEYCLOAK_REALM")
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Missing env var KEYCLOAK_REALM".into()))?;


            let decoding_key = DecodingKey::from_rsa_components(&alg.n, &alg.e)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            let mut validation = Validation::new(Algorithm::RS256);
            validation.set_audience(&allowed_audiences.iter().map(String::as_str).collect::<Vec<&str>>());
            validation.set_issuer(&[&format!("{}/realms/{}", base_url, realm)]);

            let token_data = decode::<KeycloakClaims>(token, &decoding_key, &validation)
                .map_err(|e| (StatusCode::UNAUTHORIZED, format!("JWT error: {}", e)))?;

            tracing::info!(
                "JWT verified for user with roles: {:?}",
                token_data.claims.realm_access.roles
            );

            Ok(Claims(token_data.claims))
        }
    }
}