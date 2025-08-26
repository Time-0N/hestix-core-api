// In repositories/user_repository.rs
use std::sync::Arc;
use async_trait::async_trait;
use sqlx::{Error, PgPool};
use uuid::Uuid;
use crate::model::user::UserEntity;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_subject(
        &self,
        issuer: &str,
        subject: &str,
    ) -> Result<Option<UserEntity>, Error>;

    /// Upsert a user - insert or update based on (issuer, subject)
    async fn upsert_user(
        &self,
        issuer: &str,
        subject: &str,
        username: &str,
        email: &str  // NOT optional since ZITADEL enforces it
    ) -> Result<UserEntity, Error>;

    async fn delete_by_subject(&self, issuer: &str, subject: &str) -> Result<(), Error>;
    async fn get_all_identities(&self) -> Result<Vec<(String, String)>, Error>;
    async fn get_all_users(&self) -> Result<Vec<UserEntity>, Error>;
}

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
        sqlx::query_as!(
            UserEntity,
            r#"
            SELECT * FROM users
            WHERE idp_issuer = $1 AND idp_subject = $2
            "#,
            issuer,
            subject
        )
            .fetch_optional(&*self.pool)  // Use &* like your original
            .await
    }

    async fn upsert_user(
        &self,
        issuer: &str,
        subject: &str,
        username: &str,
        email: &str
    ) -> Result<UserEntity, Error> {
        sqlx::query_as!(
            UserEntity,
            r#"
            INSERT INTO users (id, idp_issuer, idp_subject, username, email)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (idp_issuer, idp_subject)
            DO UPDATE SET
                username = EXCLUDED.username,
                email = EXCLUDED.email,
                updated_at = now()
            RETURNING *
            "#,
            Uuid::new_v4(),
            issuer,
            subject,
            username,
            email
        )
            .fetch_one(&*self.pool)  // Use &* like your original
            .await
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
            .execute(&*self.pool)  // Use &* like your original
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
            .fetch_all(&*self.pool)  // Use &* like your original
            .await?;

        Ok(rows
            .into_iter()
            .map(|r| (r.idp_issuer, r.idp_subject))
            .collect())
    }

    async fn get_all_users(&self) -> Result<Vec<UserEntity>, Error> {
        sqlx::query_as!(
            UserEntity,
            r#"
            SELECT * FROM users
            "#
        )
            .fetch_all(&*self.pool)  // Use &* like your original
            .await
    }
}