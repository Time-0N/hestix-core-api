// In repositories/user_repository.rs
use std::sync::Arc;
use async_trait::async_trait;
use sqlx::{Error, PgPool};
use uuid::Uuid;
use crate::domain::entities::User;
use crate::shared::errors::service_error::ServiceError;
use crate::domain::repositories::UserRepository as UserRepositoryTrait;

// Using domain trait instead of local duplicate

pub struct PgUserRepo {
    pool: Arc<PgPool>,
}

impl PgUserRepo {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepositoryTrait for PgUserRepo {
    async fn find_by_subject(
        &self,
        issuer: &str,
        subject: &str,
    ) -> Result<Option<User>, Error> {
        sqlx::query_as!(
            User,
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
    ) -> Result<User, Error> {
        sqlx::query_as!(
            User,
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

    // Required by domain trait
    async fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<User>, ServiceError> {
        sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE id = $1",
            id
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| ServiceError::Database(e.to_string()))
    }

    async fn find_by_issuer_and_subject(&self, issuer: &str, subject: &str) -> Result<Option<User>, ServiceError> {
        self.find_by_subject(issuer, subject)
            .await
            .map_err(|e| ServiceError::Database(e.to_string()))
    }

    async fn save(&self, user: &User) -> Result<User, ServiceError> {
        self.upsert_user(&user.idp_issuer, &user.idp_subject, &user.username, &user.email)
            .await
            .map_err(|e| ServiceError::Database(e.to_string()))
    }

    async fn update(&self, user: &User) -> Result<User, ServiceError> {
        self.save(user).await // Upsert handles both insert and update
    }

    async fn delete(&self, id: uuid::Uuid) -> Result<(), ServiceError> {
        sqlx::query!(
            "DELETE FROM users WHERE id = $1",
            id
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| ServiceError::Database(e.to_string()))?;
        Ok(())
    }

    async fn get_all_users(&self) -> Result<Vec<User>, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"
            SELECT * FROM users
            "#
        )
        .fetch_all(&*self.pool)
        .await
    }
}

// Additional helper methods for PgUserRepo
impl PgUserRepo {
    pub async fn get_all_identities(&self) -> Result<Vec<(String, String)>, Error> {
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

}