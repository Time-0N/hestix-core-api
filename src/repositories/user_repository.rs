use std::sync::Arc;
use async_trait::async_trait;
use sqlx::{Error, PgPool};

use crate::model::user::UserEntity;

/// Generic user repository using (idp_issuer, idp_subject) instead of keycloak_id.
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Look up a user by (issuer, subject).
    async fn find_by_subject(
        &self,
        issuer: &str,
        subject: &str,
    ) -> Result<Option<UserEntity>, Error>;

    /// Insert a new user.
    async fn insert(&self, user: &UserEntity) -> Result<(), Error>;

    /// Update username/email (keyed by (issuer, subject)).
    async fn update_user(&self, user: &UserEntity) -> Result<(), Error>;

    /// Delete by (issuer, subject).
    async fn delete_by_subject(&self, issuer: &str, subject: &str) -> Result<(), Error>;

    /// Return all identities (issuer, subject) pairs.
    async fn get_all_identities(&self) -> Result<Vec<(String, String)>, Error>;

    /// Return all users (full rows).
    async fn get_all_users(&self) -> Result<Vec<UserEntity>, Error>;
}

/// Postgres implementation of `UserRepository`.
pub struct PgUserRepo {
    pool: Arc<PgPool>,
}

impl PgUserRepo {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepo {
    async fn find_by_subject(
        &self,
        issuer: &str,
        subject: &str,
    ) -> Result<Option<UserEntity>, Error> {
        let rec = sqlx::query_as!(
            UserEntity,
            r#"
            SELECT
                id,
                idp_issuer,
                idp_subject,
                username,
                email,
                created_at,
                updated_at
            FROM users
            WHERE idp_issuer = $1 AND idp_subject = $2
            "#,
            issuer,
            subject
        )
            .fetch_optional(&*self.pool)
            .await?;
        Ok(rec)
    }

    async fn insert(&self, user: &UserEntity) -> Result<(), Error> {
        sqlx::query!(
            r#"
            INSERT INTO users (id, idp_issuer, idp_subject, username, email, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            user.id,
            user.idp_issuer,
            user.idp_subject,
            user.username,
            user.email,
            user.created_at,
            user.updated_at
        )
            .execute(&*self.pool)
            .await?;
        Ok(())
    }

    async fn update_user(&self, user: &UserEntity) -> Result<(), Error> {
        sqlx::query!(
            r#"
            UPDATE users
            SET username = $1,
                email    = $2,
                updated_at = $3
            WHERE idp_issuer = $4 AND idp_subject = $5
            "#,
            user.username,
            user.email,
            user.updated_at,
            user.idp_issuer,
            user.idp_subject
        )
            .execute(self.pool.as_ref())
            .await?;
        Ok(())
    }

    async fn delete_by_subject(&self, issuer: &str, subject: &str) -> Result<(), Error> {
        sqlx::query!(
            r#"
            DELETE FROM users
            WHERE idp_issuer = $1 AND idp_subject = $2
            "#,
            issuer,
            subject
        )
            .execute(&*self.pool)
            .await?;
        Ok(())
    }

    async fn get_all_identities(&self) -> Result<Vec<(String, String)>, Error> {
        let rows = sqlx::query!(
            r#"
            SELECT idp_issuer, idp_subject
            FROM users
            "#
        )
            .fetch_all(self.pool.as_ref())
            .await?;

        Ok(rows
            .into_iter()
            .map(|r| (r.idp_issuer, r.idp_subject))
            .collect())
    }

    async fn get_all_users(&self) -> Result<Vec<UserEntity>, sqlx::Error> {
        let rows = sqlx::query_as!(
            UserEntity,
            r#"
            SELECT
                id,
                idp_issuer,
                idp_subject,
                username,
                email,
                created_at,
                updated_at
            FROM users
            "#
        )
            .fetch_all(self.pool.as_ref())
            .await?;
        Ok(rows)
    }
}
