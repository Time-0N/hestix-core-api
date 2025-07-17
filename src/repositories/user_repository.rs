use std::sync::Arc;
use async_trait::async_trait;
use sqlx::{Error, PgPool};
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

    async fn update_user(&self, user: &UserEntity) -> Result<(), sqlx::Error>;

    async fn delete_by_keycloak_id(&self, keycloak_id: Uuid) -> Result<(), sqlx::Error>;

    async fn get_all_user_ids(&self) -> Result<Vec<Uuid>, sqlx::Error>;

    async fn get_all_users(&self) -> Result<Vec<UserEntity>, sqlx::Error>;
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

    async fn update_user(&self, user: &UserEntity) -> Result<(), sqlx::Error> {
        sqlx::query!(
        r#"
        UPDATE users
        SET username = $1,
            email = $2,
            updated_at = $3
        WHERE keycloak_id = $4
        "#,
        user.username,
        user.email,
        user.updated_at,
        user.keycloak_id
    )
            .execute(self.pool.as_ref())
            .await?;

        Ok(())
    }

    async fn delete_by_keycloak_id(&self, keycloak_id: Uuid) -> Result<(), Error> {
        sqlx::query!(
            r#"
            DELETE FROM users
            WHERE keycloak_id = $1
            "#,
            keycloak_id
        )
            .execute(&*self.pool)
            .await?;
        Ok(())
    }

    async fn get_all_user_ids(&self) -> Result<Vec<Uuid>, sqlx::Error> {
        let rows = sqlx::query!("SELECT keycloak_id FROM users")
            .fetch_all(self.pool.as_ref())
            .await?;

        Ok(rows.into_iter().map(|row| row.keycloak_id).collect())
    }

    async fn get_all_users(&self) -> Result<Vec<UserEntity>, sqlx::Error> {
        let rows = sqlx::query_as!(
        UserEntity,
        "SELECT * FROM users"
    )
            .fetch_all(self.pool.as_ref())
            .await?;
        Ok(rows)
    }
}
