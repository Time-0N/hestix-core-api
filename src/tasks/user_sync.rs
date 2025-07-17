use crate::app_state::AppState;

pub async fn user_sync_loop(state: AppState) {
    use tokio::time::{interval, Duration};
    let mut interval = interval(Duration::from_secs(60 * 60 * 24));

    loop {
        interval.tick().await;

        if let Err(e) = state.user_service.sync_users().await {
            tracing::error!("User sync failed: {:?}", e);
        } else {
            tracing::info!("User sync completed successfully");
        }
    }
}