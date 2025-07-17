use std::net::SocketAddr;

use anyhow::Context;
use axum::serve;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tracing_subscriber;
use tower_http::trace::TraceLayer;
use tokio::signal;

use crate::config::Config;
use crate::app_state::AppState;
use crate::routes::create_router;

async fn shutdown_signal() {
    // Wait for Ctrl+C or SIGINT
    if let Err(e) = signal::ctrl_c().await {
        tracing::error!("Failed to install shutdown signal handler: {}", e);
    }

    tracing::info!("Shutdown signal received");
}

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

    let state = AppState::new(cfg.clone(), pool.clone());
    
    // Start user sync task
    tokio::spawn({
        let state = state.clone();
        async move { 
            crate::tasks::user_sync::user_sync_loop(state).await;
        }
    });
    
    let app = create_router(state).layer(TraceLayer::new_for_http());

    let addr = {
        let host = "127.0.0.1";
        format!("{}:{}", host, cfg.port)
    }
        .parse::<SocketAddr>()
        .context("parsing listen address")?;
    tracing::info!("Listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;

    tracing::info!("Server listening on http://{}", addr);

    serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server error")?;

    Ok(())
}