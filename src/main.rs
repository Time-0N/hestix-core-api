mod bootstrap;
mod app_state;

pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod shared;

// Re-export macros at crate root
pub use shared::role::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    bootstrap::run().await
}