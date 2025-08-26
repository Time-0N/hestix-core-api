use tokio::time::{interval, Duration};
use crate::app_state::AppState;

pub async fn user_sync_loop(state: AppState) {
    // Run sync immediately on startup
    if let Err(e) = state.user_service.sync_users().await {
        tracing::error!("Initial user sync failed: {:?}", e);
    } else {
        tracing::info!("Initial user sync completed successfully");
    }

    // Then run every 24 hours
    let mut interval = interval(Duration::from_secs(60 * 60 * 24));
    interval.tick().await; // Skip the first tick since we just ran

    loop {
        interval.tick().await;

        if let Err(e) = state.user_service.sync_users().await {
            tracing::error!("Scheduled user sync failed: {:?}", e);
        } else {
            tracing::info!("Scheduled user sync completed successfully");
        }
    }
}