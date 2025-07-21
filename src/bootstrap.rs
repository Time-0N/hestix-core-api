use std::net::{SocketAddr, ToSocketAddrs};
use std::time::Duration;
use anyhow::Context;
use axum::serve;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tracing_subscriber;
use tokio::signal;
use tracing::{info, warn};
use crate::config::Config;
use crate::app_state::AppState;
use crate::middleware::security::security::{apply_security_layers};
use crate::routes::create_router;

async fn shutdown_signal() {
    // Wait for Ctrl+C or SIGINT
    if let Err(e) = signal::ctrl_c().await {
        tracing::error!("Failed to install shutdown signal handler: {}", e);
    }

    info!("Shutdown signal received");
}

fn format_display_addr(addr: &SocketAddr) -> String {
    if addr.ip().is_loopback() {
        format!("localhost:{}", addr.port())
    } else {
        addr.to_string()
    }
}

async fn try_connect_to_keycloak(cfg: &Config) -> anyhow::Result<()> {
    let url = format!(
        "{}/realms/{}/.well-known/openid-configuration",
        cfg.keycloak_base_url.trim_end_matches('/'),
        cfg.keycloak_realm,
    );
    let _ = reqwest::get(&url).await?;
    Ok(())
}

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

pub async fn run() -> anyhow::Result<()> {
    dotenv().ok();

    let cfg = Config::from_env().context("loading config")?;

    tracing_subscriber::fmt()
        .with_env_filter(&cfg.log_filter)
        .init();

    let pool = PgPoolOptions::new()
        .max_connections(cfg.db_max_connections)
        .connect(&cfg.database_url)
        .await
        .context("connecting to database")?;

    MIGRATOR.run(&pool).await.context("running migrations")?;

    loop {
        match try_connect_to_keycloak(&cfg).await {
            Ok(_) => {
                info!("Keycloak reachable");
                break;
            },
            Err(_) => {
                warn!("Keycloak not ready, retry in 2 s …");
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }

    let state = AppState::new(cfg.clone(), pool.clone());

    // Start user sync task
    tokio::spawn({
        let state = state.clone();
        async move {
            crate::tasks::user_sync::user_sync_loop(state).await;
        }
    });

    let app = apply_security_layers(create_router())
        .with_state(state);

    let addr: SocketAddr = (cfg.host.as_str(), cfg.port)
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| anyhow::anyhow!("Could not resolve address"))?;

    let listener = TcpListener::bind(addr).await?;

    info!("Server listening on http://{}", format_display_addr(&addr));

    serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server error")?;

    Ok(())
}