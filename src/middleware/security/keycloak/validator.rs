use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use crate::middleware::security::keycloak::claims::KeycloakClaims;
use crate::middleware::security::keycloak::jwk::get_cached_jwks;
use crate::middleware::security::keycloak::KeycloakError;

pub async fn validate_token_and_extract_claims(token: &str) -> Result<KeycloakClaims, KeycloakError> {
    let header = decode_header(token)
        .map_err(|e| KeycloakError::Other(format!("Invalid JWT header: {}", e)))?;
    let kid = header.kid
        .ok_or_else(|| KeycloakError::Other("Missing `kid` in JWT header".into()))?;

    let jwks = get_cached_jwks().await.map_err(KeycloakError::Other)?;

    let jwk = jwks
        .keys
        .iter()
        .find(|j| j.common.key_id.as_deref() == Some(&kid))
        .ok_or_else(|| KeycloakError::Other("Unknown `kid` in JWKs".into()))?;

    let alg = match &jwk.algorithm {
        jsonwebtoken::jwk::AlgorithmParameters::RSA(rsa) => rsa,
        _ => return Err(KeycloakError::Other("Unsupported JWK algorithm".into())),
    };

    let decoding_key = DecodingKey::from_rsa_components(&alg.n, &alg.e)
        .map_err(|e| KeycloakError::Other(e.to_string()))?;

    let allowed_audiences = std::env::var("KEYCLOAK_ALLOWED_AUDIENCES")
        .map_err(|_| KeycloakError::Other("Missing env var KEYCLOAK_ALLOWED_AUDIENCES".into()))?
        .split(',')
        .map(|s| s.trim().to_string())
        .collect::<Vec<String>>();

    let base_url = std::env::var("KEYCLOAK_BASE_URL")
        .map_err(|_| KeycloakError::Other("Missing env var KEYCLOAK_BASE_URL".into()))?;

    let realm = std::env::var("KEYCLOAK_REALM")
        .map_err(|_| KeycloakError::Other("Missing env var KEYCLOAK_REALM".into()))?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&allowed_audiences.iter().map(String::as_str).collect::<Vec<&str>>());
    validation.set_issuer(&[&format!("{}/realms/{}", base_url, realm)]);

    let token_data = decode::<KeycloakClaims>(token, &decoding_key, &validation)
        .map_err(|e| KeycloakError::Other(format!("JWT decode error: {}", e)))?;

    Ok(token_data.claims)
}
