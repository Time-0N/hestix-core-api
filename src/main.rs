use axum::{
    routing::{get, post},
    Router,
};
use tokio::net::TcpListener;
use dotenvy::dotenv;
use std::net::SocketAddr;
use std::sync::Arc;
use sqlx::postgres::PgPoolOptions;
use crate::app_state::AppState;
use crate::routes::{create_router};

mod handlers;
mod routes;
mod dto;
mod services;
mod app_state;
mod repositories;
mod models;
mod security;
mod setup;
mod macros;
mod user;
mod error;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();


    let db_url = std::env::var("DATABASE_URL").expect("MISSING DATABASE_URL");
    let db_pool = Arc::new(
        PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Could not connect to DB"),
    );

    sqlx::query("SELECT 1").execute(&*db_pool).await.expect("DB not responding");

    tracing::info!("Successfully connected and queried the DB");

    let keycloak_service = setup::keycloak::init_keycloak_service();
    tokio::spawn({
        let kc = keycloak_service.clone();
        async move {
            if kc.client.check_health().await {
                tracing::info!("Connected to Keycloak");
            } else {
                tracing::error!("Failed to connect to Keycloak");
            }
        }
    });


    let services = setup::services::init_services(db_pool.clone(), keycloak_service);

    let state = AppState {
        db: db_pool,
        auth_service: services.auth_service,
        user_service: services.user_service,
    };

    let app = create_router(state.clone());

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("PORT must be a number");

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!("Starting server on http://{}", addr);
    let listener = TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}