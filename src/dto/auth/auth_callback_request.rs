use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct AuthCallbackRequest {
    pub(crate) code: String,
}