use axum::{
    routing::{get, post},
    Router,
};
use tokio::net::TcpListener;
use dotenvy::dotenv;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::sync::Arc;
use sqlx::postgres::PgPoolOptions;
use crate::app_state::AppState;
use crate::routes::{create_router};

mod handlers;
mod routes;
mod dto;
mod services;
mod app_state;

#[tokio::main]
async fn  main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();


    let db_url = std::env::var("DATABASE_URL").expect("MISSING DATABASE_URL");
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Could not connect to DB");

    let state = AppState {
        db: Arc::new(db_pool),
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