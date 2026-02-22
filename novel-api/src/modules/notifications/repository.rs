use sqlx::PgPool;
use uuid::Uuid;

use super::models::Notification;
use crate::error::AppResult;

pub async fn list_notifications(
    pool: &PgPool,
    user_id: Uuid,
    unread_only: bool,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<Notification>> {
    let notifications = if unread_only {
        sqlx::query_as::<_, Notification>(
            "SELECT * FROM notifications WHERE user_id = $1 AND is_read = FALSE ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, Notification>(
            "SELECT * FROM notifications WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    };

    Ok(notifications)
}

pub async fn mark_as_read(pool: &PgPool, user_id: Uuid, notification_id: Uuid) -> AppResult<()> {
    sqlx::query("UPDATE notifications SET is_read = TRUE WHERE id = $1 AND user_id = $2")
        .bind(notification_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn mark_all_as_read(pool: &PgPool, user_id: Uuid) -> AppResult<()> {
    sqlx::query("UPDATE notifications SET is_read = TRUE WHERE user_id = $1 AND is_read = FALSE")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn unread_count(pool: &PgPool, user_id: Uuid) -> AppResult<i64> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND is_read = FALSE"
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(count)
}
