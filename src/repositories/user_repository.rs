use sqlx::PgPool;
use uuid::Uuid;

use crate::models::user::User;

pub async fn find_user_by_id(pool: &PgPool, user_id: Uuid) -> Result<Option<User>, sqlx::Error> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, keycloak_id, username, email
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
        .fetch_optional(pool)
        .await?;

    Ok(user)
}