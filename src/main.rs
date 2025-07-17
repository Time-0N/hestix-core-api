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
mod cache;
mod error;
mod config;
mod tasks;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    bootstrap::run().await
}