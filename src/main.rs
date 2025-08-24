mod bootstrap;
mod services;
mod app_state;
mod repositories;
mod error;
mod config;
mod util;
mod middleware;
mod http;
pub mod model;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    bootstrap::run().await
}