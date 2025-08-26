use std::collections::HashMap;
use crate::util::oidc::error::OidcError;
use crate::util::oidc::claims::OidcClaims;
use crate::util::oidc::discovery::OidcDiscovery;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Jwk {
    pub kid: String,
    pub kty: String,
    #[serde(default)]
    pub n: Option<String>,
    #[serde(default)]
    pub e: Option<String>,
    #[serde(default)]
    pub crv: Option<String>,
    #[serde(default)]
    pub x: Option<String>,
    #[serde(default)]
    pub y: Option<String>,
    #[serde(default)]
    pub alg: Option<String>,
    #[serde(rename="use", default)]
    pub use_: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwksResponse {
    pub keys: Vec<Jwk>,
}

#[derive(Clone)]
pub struct JwkCache {
    keys: HashMap<String, DecodingKey>,
}

impl JwkCache {
    pub async fn new(http_client: &Client, jwks_uri: &str) -> Result<Self, OidcError> {
        let resp = http_client.get(jwks_uri).send().await.map_err(OidcError::Network)?;
        let body = resp.error_for_status().map_err(OidcError::Network)?
            .json::<JwksResponse>().await.map_err(OidcError::Network)?;

        let mut map = HashMap::new();
        for k in body.keys {
            if let (Some(n), Some(e)) = (k.n.clone(), k.e.clone()) {
                if let Ok(dec) = DecodingKey::from_rsa_components(&n, &e) {
                    map.insert(k.kid.clone(), dec);
                }
            }
        }
        Ok(Self { keys: map })
    }

    pub fn validate(&self, token: &str, discovery: &OidcDiscovery, expected_aud: Option<&str>) -> Result<OidcClaims, OidcError> {
        let header = decode_header(token).map_err(|e| OidcError::Jwt(e.to_string()))?;
        let kid = header.kid.ok_or_else(|| OidcError::Jwt("missing kid".into()))?;
        let key = self.keys.get(&kid).ok_or_else(|| OidcError::Jwt("kid not found in JWKS".into()))?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[discovery.issuer.as_str()]);
        if let Some(aud) = expected_aud {
            validation.set_audience(&[aud]);
        }

        let data = decode::<serde_json::Value>(token, key, &validation)
            .map_err(|e| OidcError::Jwt(e.to_string()))?;
        let c = data.claims;

        let exp = c.get("exp").and_then(|v| v.as_u64()).ok_or(OidcError::MissingClaim("exp"))?;
        let iat = c.get("iat").and_then(|v| v.as_u64()).unwrap_or(0);
        let iss = c.get("iss").and_then(|v| v.as_str()).ok_or(OidcError::MissingClaim("iss"))?.to_string();
        let aud = match c.get("aud") {
            Some(serde_json::Value::String(s)) => s.clone(),
            Some(serde_json::Value::Array(arr)) => arr.get(0).and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            _ => "".to_string()
        };
        let sub = c.get("sub").and_then(|v| v.as_str()).ok_or(OidcError::MissingClaim("sub"))?.to_string();
        let email = c.get("email").and_then(|v| v.as_str()).map(|s| s.to_string());
        let preferred_username = c.get("preferred_username").and_then(|v| v.as_str()).map(|s| s.to_string());

        // Zitadel roles: { "urn:zitadel:iam:org:project:roles": { "roleA": true, ... } }
        let mut roles: Vec<String> = Vec::new();
        if let Some(obj) = c.get("urn:zitadel:iam:org:project:roles").and_then(|v| v.as_object()) {
            for (k, v) in obj {
                if v.as_bool().unwrap_or(false) {
                    roles.push(k.clone());
                }
            }
        }

        Ok(OidcClaims { exp, iat, iss, aud, sub, email, preferred_username, roles })
    }
}
