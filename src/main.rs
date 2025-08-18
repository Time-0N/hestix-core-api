mod bootstrap;
mod handlers;
mod routes;
mod dto;
mod services;
mod app_state;
mod repositories;
mod models;
mod macros;
mod cache;
mod error;
mod config;
mod tasks;
mod middleware;
mod oidc;
mod providers;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    bootstrap::run().await
}