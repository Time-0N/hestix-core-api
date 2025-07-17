use std::env;

#[derive(Clone)]
pub struct KeycloakConfig {
    pub base_url: String,
    pub realm: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}