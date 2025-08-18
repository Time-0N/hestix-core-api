use serde::Deserialize;
use crate::oidc::error::OidcError;

#[derive(Debug, Clone, Deserialize)]
pub struct OidcDiscovery {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub jwks_uri: String,
    #[serde(default)]
    pub userinfo_endpoint: Option<String>,
    #[serde(default)]
    pub end_session_endpoint: Option<String>,
}

impl OidcDiscovery {
    pub async fn fetch(issuer_url: &str) -> Result<Self, OidcError> {
        let well_known = if issuer_url.ends_with("/.well-known/openid-configuration") {
            issuer_url.to_string()
        } else {
            format!("{}/.well-known/openid-configuration", issuer_url.trim_end_matches('/'))
        };
        let resp = reqwest::get(&well_known).await.map_err(OidcError::Network)?;
        let doc = resp.error_for_status().map_err(OidcError::Network)?
            .json::<OidcDiscovery>().await.map_err(OidcError::Network)?;
        Ok(doc)
    }
}
