use std::net::{SocketAddr, ToSocketAddrs};
use std::time::Duration;
use anyhow::Context;
use axum::serve;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tracing_subscriber;
use tokio::signal;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};
use crate::config::Config;
use crate::app_state::AppState;
use crate::middleware::security::security_layer::{apply_security_layers};
use crate::http::routes::create_router;
use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

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

async fn wait_for_oidc_issuer(cfg: &Config) -> anyhow::Result<()> {
    let url = format!(
        "{}/.well-known/openid-configuration",
        cfg.issuer_url.trim_end_matches('/')
    );
    loop {
        match reqwest::get(&url).await {
            Ok(resp) if resp.status().is_success() => {
                info!("OIDC issuer reachable");
                return Ok(());
            }
            _ => {
                warn!("OIDC issuer not ready, retry in 2 s …");
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }
}

pub fn init_tracing(cfg_filter: Option<&str>) {
    let filter = if std::env::var_os("RUST_LOG").is_some() {
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("hestix_core_api=info,tower_http=info"))
    } else if let Some(s) = cfg_filter.filter(|s| !s.is_empty()) {
        EnvFilter::new(s.to_string())
    } else {
        EnvFilter::new("hestix_core_api=info,tower_http=info")
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().compact()) // nice console logs
        .init();
}

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

pub async fn run() -> anyhow::Result<()> {
    dotenv().ok();

    let cfg = Config::from_env().context("loading config")?;

    init_tracing(Some(&cfg.log_filter));

    let pool = PgPoolOptions::new()
        .max_connections(cfg.db_max_connections)
        .connect(&cfg.database_url)
        .await
        .context("connecting to database")?;

    MIGRATOR.run(&pool).await.context("running migrations")?;

    wait_for_oidc_issuer(&cfg).await?;

    let provider = std::sync::Arc::new(
        crate::util::oidc::providers::zitadel::provider::ZitadelProvider::new(
            &cfg.issuer_url,
            &cfg.client_id,
            &cfg.client_secret,
            &cfg.redirect_url,
            &cfg.scopes,
        ).await?
    );

    let state = AppState::new(cfg.clone(), pool.clone(), provider.clone());

    // Start user sync task
    tokio::spawn({
        let state = state.clone();
        async move {
            crate::util::tasks::user_sync::user_sync_loop(state).await;
        }
    });

    let app = apply_security_layers(create_router())
        .layer(TraceLayer::new_for_http())
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