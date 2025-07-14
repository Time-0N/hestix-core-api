use std::sync::Arc;
use std::time::Duration;
use moka::future::Cache;
use uuid::Uuid;
use crate::user::user::User;

pub type UserCache = Cache<Uuid, Arc<User>>;

pub fn new_user_cache() -> UserCache {
    Cache::builder()
        .time_to_live(Duration::from_secs(600))
        .max_capacity(10_000)
        .build()
}