use crate::util::oidc::OidcError;
use crate::util::oidc::provider::{IdpUser, OidcAdminApi};
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
struct UseItem {
    id: String,
    #[serde(rename = "preferredLoginName")]
    preferred_login_name: Option<String>,
    email: Option<String>,
    #[serde(rename = "userName")]
    user_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ListUsersResponse {
    result: Vec<UseItem>,
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
}

#[async_trait]
impl OidcAdminApi for ZitadelAdminApi {
    async fn fetch_all_users(&self) -> Result<Vec<IdpUser>, OidcError> {
        let mut acc = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let mut req = self.http_client
                .get(self.base_url.join("/management/v1/users").unwrap())
                .bearer_auth(&self.token)
                .query(&[("page_size", "500")]);

            if let Some(t) = &page_token {
                req = req.query(&[("page_token", t)]);
            }

            let body: ListUsersResponse = req
                .send().await.map_err(OidcError::Network)?
                .error_for_status().map_err(OidcError::Network)?
                .json().await.map_err(OidcError::Network)?;

            for u in body.result {
                acc.push(IdpUser {
                    idp_subject:  u.id,
                    email:    u.email,
                    username: u.preferred_login_name.or(u.user_name),
                });
            }

            page_token = body.next_page_token;
            if page_token.is_none() { break; }
        }

        Ok(acc)
    }
}
