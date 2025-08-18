use crate::oidc::OidcError;
use crate::oidc::provider::{IdpUser, OidcAdminApi};
use async_trait::async_trait;

pub struct ZitadelAdminApi {
    pub base_url: String,
    pub token: String,
}

impl ZitadelAdminApi {
    pub fn new(base_url: String, token: String) -> Self {
        Self { base_url, token }
    }
}

#[async_trait]
impl OidcAdminApi for ZitadelAdminApi {
    async fn fetch_all_users(&self) -> Result<Vec<IdpUser>, OidcError> {
        // TODO: call Zitadel mgmt API (not needed for login flow)
        Ok(vec![])
    }
}
