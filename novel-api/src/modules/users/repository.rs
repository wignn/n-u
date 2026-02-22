use sqlx::PgPool;
use uuid::Uuid;

use super::models::{User, UserProfile};
use crate::error::{AppError, AppResult};

pub async fn create_user(
    pool: &PgPool,
    username: &str,
    email: &str,
    password_hash: &str,
    display_name: Option<&str>,
) -> AppResult<User> {
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (username, email, password_hash, display_name) VALUES ($1, $2, $3, $4) RETURNING *"
    )
    .bind(username)
    .bind(email)
    .bind(password_hash)
    .bind(display_name)
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref db_err) if db_err.constraint().is_some() => {
            AppError::Conflict("Username or email already exists".to_string())
        }
        _ => AppError::Database(e),
    })?;

    Ok(user)
}

pub async fn find_by_email(pool: &PgPool, email: &str) -> AppResult<Option<User>> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await?;
    Ok(user)
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> AppResult<Option<User>> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(user)
}

pub async fn get_profile(pool: &PgPool, id: Uuid) -> AppResult<UserProfile> {
    let profile = sqlx::query_as!(
        UserProfile,
        "SELECT id, username, display_name, avatar_url, bio, created_at FROM users WHERE id = $1",
        id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(profile)
}

pub async fn update_profile(
    pool: &PgPool,
    user_id: Uuid,
    display_name: Option<&str>,
    avatar_url: Option<&str>,
    bio: Option<&str>,
) -> AppResult<User> {
    let user = sqlx::query_as::<_, User>(
        "UPDATE users SET display_name = COALESCE($2, display_name), avatar_url = COALESCE($3, avatar_url), bio = COALESCE($4, bio) WHERE id = $1 RETURNING *"
    )
    .bind(user_id)
    .bind(display_name)
    .bind(avatar_url)
    .bind(bio)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn update_last_login(pool: &PgPool, user_id: Uuid) -> AppResult<()> {
    sqlx::query("UPDATE users SET last_login_at = NOW() WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}
