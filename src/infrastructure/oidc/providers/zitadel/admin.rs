use crate::infrastructure::oidc::OidcError;
use crate::infrastructure::oidc::provider::{IdpUser, OidcAdminApi};
use serde::Deserialize;
use async_trait::async_trait;
use reqwest::{Client, Url};

pub struct ZitadelAdminApi {
    http_client: Client,
    base_url: Url,
    token: String,
}

impl ZitadelAdminApi {
    pub fn new(http: Client, base_url: &str, bearer_token: String) -> Result<Self, url::ParseError> {
        Ok(Self { http_client: http, base_url: Url::parse(base_url)?, token: bearer_token })
    }
}

#[derive(Debug, Deserialize)]
struct UserV2 {
    #[serde(rename = "userId")]
    user_id: String,
    #[serde(rename = "preferredLoginName")]
    preferred_login_name: Option<String>,
    #[serde(rename = "loginNames")]
    login_names: Option<Vec<String>>,
    #[serde(rename = "human")]
    human_user: Option<HumanUser>,
}

#[derive(Debug, Deserialize)]
struct HumanUser {
    #[serde(rename = "email")]
    email: Option<Email>,
}

#[derive(Debug, Deserialize)]
struct Email {
    email: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ListUsersV2Response {
    result: Vec<UserV2>,
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
}

#[async_trait]
impl OidcAdminApi for ZitadelAdminApi {
    async fn fetch_all_users(&self) -> Result<Vec<IdpUser>, OidcError> {
        let mut acc = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            // Build request body for v2 API
            let mut request_body = serde_json::json!({
                "queries": [],
                "sorting_column": "SORTING_COLUMN_CREATION_DATE",
                "asc": true
            });

            if let Some(t) = &page_token {
                request_body["page_token"] = serde_json::Value::String(t.clone());
            }

            let req = self.http_client
                .post(self.base_url.join("/v2/users").unwrap())
                .bearer_auth(&self.token)
                .header("Content-Type", "application/json")
                .json(&request_body);

            let body: ListUsersV2Response = req
                .send().await.map_err(OidcError::Network)?
                .error_for_status().map_err(OidcError::Network)?
                .json().await.map_err(OidcError::Network)?;

            for u in body.result {
                // Skip service/machine users - only sync human users
                if u.human_user.is_none() {
                    continue;
                }

                // Extract email from nested structure
                let email = u.human_user
                    .as_ref()
                    .and_then(|h| h.email.as_ref())
                    .and_then(|e| e.email.clone());

                // Use preferred login name, or first login name as fallback
                let username = u.preferred_login_name
                    .or_else(|| u.login_names.as_ref()
                        .and_then(|names| names.first().cloned()));

                acc.push(IdpUser {
                    idp_subject: u.user_id,
                    email,
                    username,
                });
            }

            page_token = body.next_page_token;
            if page_token.is_none() { break; }
        }

        Ok(acc)
    }
}
