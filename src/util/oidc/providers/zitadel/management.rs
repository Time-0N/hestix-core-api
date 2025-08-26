// src/util/oidc/providers/zitadel/management.rs
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{info, debug};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Deserialize)]
pub struct ServiceAccountKey {
    #[serde(rename = "type")]
    pub key_type: String,
    #[serde(rename = "keyId")]
    pub key_id: String,
    pub key: String,  // Private key in PEM format
    #[serde(rename = "userId")]
    pub user_id: String,
}

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Debug, Deserialize)]
pub struct ZitadelUser {
    pub id: String,
    #[serde(rename = "userName")]
    pub username: Option<String>,
    pub state: String,  // ACTIVE, INACTIVE, etc.
    #[serde(rename = "preferredLoginName")]
    pub preferred_login_name: Option<String>,
    pub human: Option<HumanUser>,
    pub machine: Option<MachineUser>,
}

#[derive(Debug, Deserialize)]
pub struct HumanUser {
    pub profile: UserProfile,
    pub email: EmailInfo,
}

#[derive(Debug, Deserialize)]
pub struct MachineUser {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct UserProfile {
    #[serde(rename = "givenName")]
    pub given_name: Option<String>,
    #[serde(rename = "familyName")]
    pub family_name: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EmailInfo {
    pub email: String,
    #[serde(rename = "isVerified")]
    pub is_verified: bool,
}

#[derive(Debug, Deserialize)]
pub struct ListUsersResponse {
    pub result: Vec<ZitadelUser>,
    pub details: ListDetails,
}

#[derive(Debug, Deserialize)]
pub struct ListDetails {
    #[serde(rename = "totalResult")]
    pub total_result: String,
}

pub struct ZitadelManagementClient {
    client: Client,
    api_url: String,
    service_key: ServiceAccountKey,
    access_token: Option<String>,
    token_expiry: Option<u64>,
}

impl ZitadelManagementClient {
    pub fn new(client: Client, api_url: String, service_key_json: &str) -> Result<Self> {
        let service_key: ServiceAccountKey = serde_json::from_str(service_key_json)?;

        info!("Initialized ZITADEL Management client for user {}", service_key.user_id);

        Ok(Self {
            client,
            api_url: api_url.trim_end_matches('/').to_string(),
            service_key,
            access_token: None,
            token_expiry: None,
        })
    }

    /// Get or refresh access token for service account
    async fn ensure_access_token(&mut self) -> Result<String> {
        // Check if we have a valid token
        if let Some(token) = &self.access_token {
            if let Some(expiry) = self.token_expiry {
                let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
                if now < expiry - 60 {  // 1 minute buffer
                    return Ok(token.clone());
                }
            }
        }

        debug!("Refreshing ZITADEL service account token");

        // Create JWT assertion
        let jwt = self.create_jwt_assertion()?;

        // Exchange for access token
        let token_endpoint = format!("{}/oauth/v2/token", self.api_url);

        let params = [
            ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
            ("assertion", &jwt),
            ("scope", "openid profile email urn:zitadel:iam:org:project:id:zitadel:aud"),
        ];

        let response = self.client
            .post(&token_endpoint)
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to get access token: {}", error_text);
        }

        let token_response: TokenResponse = response.json().await?;

        // Store token and expiry
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        self.access_token = Some(token_response.access_token.clone());
        self.token_expiry = Some(now + token_response.expires_in as u64);

        Ok(token_response.access_token)
    }

    fn create_jwt_assertion(&self) -> Result<String> {
        use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();

        #[derive(Serialize)]
        struct Claims {
            iss: String,
            sub: String,
            aud: Vec<String>,
            iat: u64,
            exp: u64,
        }

        let claims = Claims {
            iss: self.service_key.user_id.clone(),
            sub: self.service_key.user_id.clone(),
            aud: vec![self.api_url.clone()],
            iat: now,
            exp: now + 3600,  // 1 hour
        };

        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(self.service_key.key_id.clone());

        let key = EncodingKey::from_rsa_pem(self.service_key.key.as_bytes())?;

        Ok(encode(&header, &claims, &key)?)
    }

    /// List all active human users in the organization
    pub async fn list_all_users(&mut self) -> Result<Vec<(String, String, String, String)>> {
        let token = self.ensure_access_token().await?;

        let mut all_users = Vec::new();
        let mut offset = 0;
        let limit = 100;

        loop {
            let url = format!("{}/management/v1/users/_search", self.api_url);

            let body = serde_json::json!({
                "query": {
                    "offset": offset,
                    "limit": limit,
                    "asc": true
                },
                // Only get human users (not service accounts)
                "queries": [
                    {
                        "humanQuery": {}
                    }
                ]
            });

            let response = self.client
                .post(&url)
                .bearer_auth(&token)
                .json(&body)
                .send()
                .await?;

            if !response.status().is_success() {
                let error_text = response.text().await?;
                anyhow::bail!("Failed to list users: {}", error_text);
            }

            let list_response: ListUsersResponse = response.json().await?;
            let user_count = list_response.result.len();

            for user in list_response.result {
                // Skip inactive users and service accounts
                if user.state != "USER_STATE_ACTIVE" || user.human.is_none() {
                    continue;
                }

                if let Some(human) = user.human {
                    let username = user.preferred_login_name
                        .or(user.username)
                        .unwrap_or_else(|| user.id.clone());

                    let subject = user.id;
                    let email = human.email.email;
                    let issuer = self.api_url.clone();

                    all_users.push((issuer, subject, username, email));
                }
            }

            if user_count < limit {
                break;  // No more users
            }

            offset += limit;
        }

        info!("Fetched {} active users from ZITADEL", all_users.len());
        Ok(all_users)
    }
}