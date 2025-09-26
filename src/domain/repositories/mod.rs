use async_trait::async_trait;
use crate::domain::entities::User;
use crate::shared::errors::service_error::ServiceError;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<User>, ServiceError>;
    async fn find_by_issuer_and_subject(&self, issuer: &str, subject: &str) -> Result<Option<User>, ServiceError>;
    async fn find_by_subject(&self, issuer: &str, subject: &str) -> Result<Option<User>, sqlx::Error>;
    async fn save(&self, user: &User) -> Result<User, ServiceError>;
    async fn update(&self, user: &User) -> Result<User, ServiceError>;
    async fn delete(&self, id: uuid::Uuid) -> Result<(), ServiceError>;
    async fn upsert_user(&self, issuer: &str, subject: &str, username: &str, email: &str) -> Result<User, sqlx::Error>;
    async fn delete_by_subject(&self, issuer: &str, subject: &str) -> Result<(), sqlx::Error>;
    async fn get_all_users(&self) -> Result<Vec<User>, sqlx::Error>;
}