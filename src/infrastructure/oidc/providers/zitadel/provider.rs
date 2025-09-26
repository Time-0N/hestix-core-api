use async_trait::async_trait;
use reqwest::{Client, Url};
use crate::infrastructure::oidc::{OidcClaims, OidcError, discovery::OidcDiscovery, jwk::JwkCache, provider::OidcProvider, RoleMapper};
use crate::application::dto::auth::token_response::TokenResponse;
use base64::{engine::general_purpose, Engine as _};
use serde_json::Value;
use crate::infrastructure::oidc::providers::zitadel::role_mapper::ZitadelRoleMapper;

pub struct ZitadelProvider {
    http_client: Client,
    client_id: String,
    redirect_url: String,
    scopes: String,
    discovery: OidcDiscovery,
    jwks: JwkCache,
}

impl ZitadelProvider {
    pub async fn new(
        http_client: Client,
        issuer_url: &str,
        client_id: &str,
        redirect_url: &str,
        scopes: &str,
    ) -> Result<Self, OidcError> {
        let discovery = OidcDiscovery::fetch(&http_client,issuer_url).await?;
        let jwks = JwkCache::new(&http_client,&discovery.jwks_uri).await?;

        Ok(Self {
            http_client,
            client_id: client_id.to_string(),
            redirect_url: redirect_url.to_string(),
            scopes: scopes.to_string(),
            discovery,
            jwks,
        })
    }
}

fn decode_jwt_payload(token: &str) -> Result<Value, OidcError> {
    let payload_b64 = token.split('.').nth(1).ok_or_else(|| OidcError::Jwt("malformed JWT".into()))?;
    let bytes = general_purpose::URL_SAFE_NO_PAD
        .decode(payload_b64)
        .map_err(|e| OidcError::Jwt(format!("payload b64 decode: {e}")))?;
    serde_json::from_slice::<Value>(&bytes).map_err(OidcError::Json)
}


#[async_trait]
impl OidcProvider for ZitadelProvider {
    async fn authorize_url(&self, state: Option<String>, code_challenge: Option<&str>) -> String {
        let mut url = Url::parse(&self.discovery.authorization_endpoint)
            .expect("invalid authorization_endpoint");
        {
            let mut qp = url.query_pairs_mut();
            qp.append_pair("client_id", &self.client_id);
            qp.append_pair("response_type", "code");
            qp.append_pair("redirect_uri", &self.redirect_url);
            qp.append_pair("scope", &self.scopes);
            if let Some(s) = state { qp.append_pair("state", &s); }
            if let Some(ch) = code_challenge {
                qp.append_pair("code_challenge", ch);
                qp.append_pair("code_challenge_method", "S256");
            }
            qp.append_pair("response_mode", "query");
        }
        url.to_string()
    }

    async fn exchange_code_for_tokens(&self, code: &str, code_verifier: Option<&str>) -> Result<TokenResponse, OidcError> {
        let mut form = vec![
            ("grant_type", "authorization_code".to_string()),
            ("code", code.to_string()),
            ("redirect_uri", self.redirect_url.clone()),
            ("client_id", self.client_id.clone()),
        ];
        if let Some(v) = code_verifier {
            form.push(("code_verifier".into(), v.to_string()));
        }

        let resp = self.http_client
            .post(&self.discovery.token_endpoint)
            .form(&form)
            .send()
            .await
            .map_err(OidcError::Network)?
            .error_for_status()
            .map_err(OidcError::Network)?;
        let tr = resp.json::<TokenResponse>().await.map_err(OidcError::Network)?;
        Ok(tr)
    }

    async fn refresh_access_token(&self, refresh_token: &str) -> Result<TokenResponse, OidcError> {
        let mut form = vec![
            ("grant_type", "refresh_token".to_string()),
            ("refresh_token", refresh_token.to_string()),
            ("client_id", self.client_id.clone()),
        ];

        let resp = self.http_client
            .post(&self.discovery.token_endpoint)
            .form(&form)
            .send()
            .await
            .map_err(OidcError::Network)?
            .error_for_status()
            .map_err(OidcError::Network)?;
        let tr = resp.json::<TokenResponse>().await.map_err(OidcError::Network)?;
        Ok(tr)
    }

    async fn validate_access_token(&self, token: &str) -> Result<OidcClaims, OidcError> {
        // 1) Verify signature & standard claims (exp/aud/iss/…)
        let mut claims = self
            .jwks
            .validate(token, &self.discovery, Some(&self.client_id))?;

        // 2) Read provider-specific fields from the *raw* payload
        let raw = decode_jwt_payload(token)?;
        let mapper = ZitadelRoleMapper;
        claims.roles = mapper.extract_roles(&raw);

        Ok(claims)
    }

    async fn validate_id_token(&self, id_token: &str) -> Result<OidcClaims, OidcError> {
        // Reuse your JWKS validator but force the audience to client_id
        self.jwks.validate(id_token, &self.discovery, Some(&self.client_id))
    }
}
