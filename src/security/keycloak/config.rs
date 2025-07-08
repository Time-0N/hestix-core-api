use std::env;

#[derive(Clone)]
pub struct KeycloakConfig {
    pub base_url: String,
    pub realm: String,
    pub client_id: String,
    pub client_secret: String,
}

impl KeycloakConfig {
    pub fn from_env() -> Self {
        KeycloakConfig {
            base_url: env::var("KEYCLOAK_BASE_URL").expect("KEYCLOAK_BASE_URL not set"),
            realm: env::var("KEYCLOAK_REALM").expect("KEYCLOAK_REALM not set"),
            client_id: env::var("KEYCLOAK_CLIENT_ID").expect("KEYCLOAK_CLIENT_ID not set"),
            client_secret: env::var("KEYCLOAK_CLIENT_SECRET").expect("KEYCLOAK_CLIENT_SECRET not set"),
        }
    }
}