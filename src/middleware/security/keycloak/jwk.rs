use std::sync::RwLock;
use jsonwebtoken::jwk::JwkSet;
use once_cell::sync::Lazy;

static CACHED_JWKS: Lazy<RwLock<Option<JwkSet>>> = Lazy::new(|| RwLock::new(None));

pub async fn get_cached_jwks() -> Result<JwkSet, String> {
    {
        let guard = CACHED_JWKS.read().unwrap();
        if let Some(cached) = &*guard {
            return Ok(cached.clone());
        }
    }

    let url = format!(
        "{}/realms/{}/protocol/openid-connect/certs",
        std::env::var("KEYCLOAK_BASE_URL").unwrap(),
        std::env::var("KEYCLOAK_REALM").unwrap()
    );

    let res = reqwest::get(&url)
        .await
        .map_err(|e| format!("Failed to fetch JWKs: {}", e))?;

    let jwks: JwkSet = res.json().await.map_err(|e| format!("Invalid JWK JSON: {}", e))?;

    let mut guard = CACHED_JWKS.write().unwrap();
    *guard = Some(jwks.clone());

    Ok(jwks)
}