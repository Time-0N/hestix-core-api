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
    pub allowed_origin: String,

    pub issuer_url: String,
    pub client_id: String,
    pub redirect_url: String,
    pub scopes: String,

    pub zitadel_service_token: Option<String>,
    pub environment: String,
}

impl Config {
    pub fn is_production(&self) -> bool {
        self.environment.to_lowercase() == "production"
    }

    pub fn is_development(&self) -> bool {
        self.environment.to_lowercase() == "development"
    }
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
        let issuer_url = env::var("OIDC_ISSUER_URL")
            .context("OIDC_ISSUER_URL must be set")?;
        let client_id = env::var("OIDC_CLIENT_ID")
            .context("OIDC_CLIENT_ID must be set")?;
        let redirect_url = env::var("OIDC_REDIRECT_URL")
            .context("OIDC_REDIRECT_URL must be set")?;
        let scopes = env::var("OIDC_SCOPES")
            .context("OIDC_SCOPES must be set")?;

        let db_max_connections = env::var("DB_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<u32>()
            .context("DB_MAX_CONNECTIONS must be a positive integer")?;
        let log_filter = env::var("LOG_FILTER")
            .unwrap_or_else(|_| "info".to_string());
        let allowed_origin = env::var("CORS_ALLOWED_ORIGIN")
            .unwrap_or_else(|_| "info".to_string());
        let environment = env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string());

        let zitadel_service_token = std::env::var("ZITADEL_SERVICE_TOKEN").ok()
            .or_else(|| {
                std::env::var("ZITADEL_SERVICE_TOKEN_PATH").ok()
                    .and_then(|path| {
                        match std::fs::read_to_string(&path) {
                            Ok(content) => {
                                tracing::info!("Loaded ZITADEL service token from {}", path);
                                Some(content.trim().to_string())
                            }
                            Err(e) => {
                                tracing::warn!("Failed to load service key from {}: {}", path, e);
                                None
                            }
                        }
                    })
            });

        Ok(Config {
            database_url,
            db_max_connections,
            host,
            port,
            log_filter,
            allowed_origin,
            issuer_url,
            client_id,
            redirect_url,
            scopes,
            zitadel_service_token,
            environment,
        })
    }
}