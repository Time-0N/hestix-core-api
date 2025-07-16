use std::sync::Arc;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::user::UserEntity;

/// Defines all cache‐related data operations.
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Look up a cache by their Keycloak ID.
    async fn find_by_keycloak_id(
        &self,
        keycloak_id: Uuid,
    ) -> Result<Option<UserEntity>, sqlx::Error>;

    /// Insert a new cache record.
    async fn insert(&self, user: &UserEntity) -> Result<(), sqlx::Error>;
}

/// Postgres implementation of `UserRepo`.
pub struct PgUserRepo {
    pool: Arc<PgPool>,
}

impl PgUserRepo {
    /// Construct a new PgUserRepo backed by the given pool.
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepo {
    async fn find_by_keycloak_id(
        &self,
        keycloak_id: Uuid,
    ) -> Result<Option<UserEntity>, sqlx::Error> {
        let rec = sqlx::query_as!(
            UserEntity,
            r#"
            SELECT id, keycloak_id, username, email, created_at, updated_at
            FROM users
            WHERE keycloak_id = $1
            "#,
            keycloak_id
        )
            .fetch_optional(&*self.pool)
            .await?;
        Ok(rec)
    }

    async fn insert(&self, user: &UserEntity) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO users (id, keycloak_id, username, email, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            user.id,
            user.keycloak_id,
            user.username,
            user.email,
            user.created_at,
            user.updated_at
        )
            .execute(&*self.pool)
            .await?;
        Ok(())
    }
}
