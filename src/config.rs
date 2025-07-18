use std::env;
use anyhow::Context;
use dotenvy::dotenv;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub database_url: String,
    pub db_max_connections: u32,
    pub host: String,
    pub port: u16,
    pub log_filter: String,

    pub keycloak_base_url: String,
    pub keycloak_realm: String,
    pub keycloak_client_id: String,
    pub keycloak_client_secret: String,
    pub keycloak_redirect_uri: String,
}

impl Config {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .context("DATABASE_URL must be set")?;
        let host = env::var("HOST")
            .context("HOST must be set")?;
        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".into())
            .parse::<u16>()
            .context("PORT must be a valid port number")?;
        let keycloak_base_url = env::var("KEYCLOAK_BASE_URL")
            .context("KEYCLOAK_BASE_URL must be set")?;
        let keycloak_realm = env::var("KEYCLOAK_REALM")
            .context("KEYCLOAK_REALM must be set")?;
        let keycloak_client_id = env::var("KEYCLOAK_CLIENT_ID")
            .context("KEYCLOAK_CLIENT_ID must be set")?;
        let keycloak_client_secret = env::var("KEYCLOAK_CLIENT_SECRET")
            .context("KEYCLOAK_CLIENT_SECRET must be set")?;
        let keycloak_redirect_uri = env::var("KEYCLOAK_REDIRECT_URI")
            .context("KEYCLOAK_REDIRECT_URI must be set")?;

        let db_max_connections = env::var("DB_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<u32>()
            .context("DB_MAX_CONNECTIONS must be a positive integer")?;
        let log_filter = env::var("LOG_FILTER")
            .unwrap_or_else(|_| "info".to_string());

        Ok(Config {
            database_url,
            db_max_connections,
            host,
            port,
            log_filter,
            keycloak_base_url,
            keycloak_realm,
            keycloak_client_id,
            keycloak_client_secret,
            keycloak_redirect_uri,
        })
    }
}