use std::sync::Arc;
use crate::security::keycloak::client::KeycloakClient;
use crate::security::keycloak::config::KeycloakConfig;
use crate::services::keycloak_service::KeycloakService;

pub fn init_keycloak_service() -> Arc<KeycloakService> {
    let config = KeycloakConfig::from_env();
    let client = KeycloakClient::new(config.clone());
    Arc::new(KeycloakService::new(client))
}
