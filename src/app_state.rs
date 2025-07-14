use std::sync::Arc;
use sqlx::PgPool;
use crate::services::auth_service::AuthService;
use crate::user::user_service::UserService;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<PgPool>,
    pub auth_service: Arc<AuthService>,
    pub user_service: Arc<UserService>,
}
