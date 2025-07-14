use sqlx::PgPool;
use uuid::Uuid;
use crate::user::user::User;

pub async fn find_user_by_keycloak_id(pool: &PgPool, keycloak_id: Uuid) -> Result<Option<User>, sqlx::Error> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, keycloak_id, username, email
        FROM users
        WHERE keycloak_id = $1
        "#,
        keycloak_id
    )
        .fetch_optional(pool)
        .await?;

    Ok(user)
}

pub async fn insert_user(pool: &PgPool, user: &User) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO users (id, keycloak_id, username, email)
        VALUES ($1, $2, $3, $4)
        "#,
        user.id,
        user.keycloak_id,
        user.username,
        user.email
    )
        .execute(pool)
        .await?;

    Ok(())
}