mod bootstrap;
mod handlers;
mod routes;
mod dto;
mod services;
mod app_state;
mod repositories;
mod models;
mod security;
mod macros;
mod user;
mod error;
mod config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    bootstrap::run().await
}