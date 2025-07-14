use sqlx::PgPool;
use uuid::Uuid;
use crate::models::user::UserEntity;

pub async fn find_user_by_keycloak_id(pool: &PgPool, keycloak_id: Uuid) -> Result<Option<UserEntity>, sqlx::Error> {
    let user = sqlx::query_as!(
        UserEntity,
        r#"
        SELECT id, keycloak_id, username, email, created_at, updated_at
        FROM users
        WHERE keycloak_id = $1
        "#,
        keycloak_id
    )
        .fetch_optional(pool)
        .await?;

    Ok(user)
}

pub async fn insert_user(pool: &PgPool, user: &UserEntity) -> Result<(), sqlx::Error> {
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
        .execute(pool)
        .await?;

    Ok(())
}