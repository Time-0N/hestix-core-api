use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct UserEntity {
    pub id: uuid::Uuid,
    pub idp_issuer: String,
    pub idp_subject: String,
    pub username: String,
    pub email: String,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}