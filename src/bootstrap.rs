use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::time::Duration;
use anyhow::Context;
use axum::extract::DefaultBodyLimit;
use axum::serve;
use dotenvy::dotenv;
use reqwest::Client;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tracing_subscriber;
use tokio::signal;
use tokio::sync::Mutex;
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use tracing::{info, warn, Level};
use crate::infrastructure::config::Config;
use crate::app_state::AppState;
use crate::shared::middleware::apply_security_layers;
use crate::infrastructure::web::routes::create_router;
use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use crate::infrastructure::web::client::build_http_client;
use crate::infrastructure::oidc::providers::zitadel::management::ZitadelManagementClient;

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

async fn wait_for_oidc_issuer(http_client: &Client, cfg: &Config) -> anyhow::Result<()> {
    let url = format!(
        "{}/.well-known/openid-configuration",
        cfg.issuer_url.trim_end_matches('/')
    );
    loop {
        match http_client.get(url.as_str()).send().await {
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

    info!("Booting with environment: {}", cfg.environment);

    let pool = PgPoolOptions::new()
        .max_connections(cfg.db_max_connections)
        .connect(&cfg.database_url)
        .await
        .context("connecting to database")?;

    MIGRATOR.run(&pool).await.context("running migrations")?;

    let http_client: Client = build_http_client(
        &format!("hestix-core/{}", env!("CARGO_PKG_VERSION")),
        std::env::var("HTTP_ACCEPT_INVALID_CERTS")
            .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE"))
            .unwrap_or(false),
        10,                            // redirects
        Duration::from_secs(5),        // connect_timeout
        Duration::from_secs(15),       // request timeout
    )?;

    wait_for_oidc_issuer(&http_client, &cfg).await?;

    let provider = std::sync::Arc::new(
        crate::infrastructure::oidc::providers::zitadel::provider::ZitadelProvider::new(
            http_client.clone(),
            &cfg.issuer_url,
            &cfg.client_id,
            &cfg.redirect_url,
            &cfg.scopes,
        ).await?
    );

    let management_client = if let Some(service_key) = &cfg.zitadel_service_key {
        match ZitadelManagementClient::new(
            http_client.clone(),
            cfg.issuer_url.clone(),
            service_key
        ) {
            Ok(client) => {
                info!("ZITADEL Management API client initialized");
                Some(Arc::new(Mutex::new(client)))
            }
            Err(e) => {
                warn!("Failed to initialize ZITADEL Management API: {}", e);
                warn!("User sync will only refresh from database");
                None
            }
        }
    } else {
        info!("ZITADEL Management API not configured - user sync will only refresh from database");
        None
    };


    // Cast management client to trait object
    let management_client_trait: Option<Arc<Mutex<dyn crate::infrastructure::oidc::provider::OidcAdminApi + Send + Sync>>> =
        management_client.map(|mc| mc as Arc<Mutex<dyn crate::infrastructure::oidc::provider::OidcAdminApi + Send + Sync>>);

    let state = AppState::new(cfg.clone(), pool.clone(), provider.clone(), http_client, management_client_trait);

    // Start user sync task only if ZITADEL service key is configured
    if cfg.zitadel_service_key.is_some() {
        info!("ZITADEL_SERVICE_KEY configured - starting user sync job");
        tokio::spawn({
            let state = state.clone();
            async move {
                crate::application::user_sync::user_sync_loop(state).await;
            }
        });
    } else {
        warn!("ZITADEL_SERVICE_KEY not set, skipping start of user sync job");
    }

    let app = apply_security_layers(create_router())
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024))
        .layer(
            TraceLayer::new_for_http()
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(tower_http::LatencyUnit::Millis)
                        .include_headers(false)
                )
        )
        .with_state(state);

    let addr: SocketAddr = (cfg.host.as_str(), cfg.port)
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| anyhow::anyhow!("Could not resolve address"))?;

    let listener = TcpListener::bind(addr).await?;

    info!("Server listening on {}", format_display_addr(&addr));

    serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server error")?;

    Ok(())
}