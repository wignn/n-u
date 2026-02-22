use sqlx::PgPool;
use uuid::Uuid;

use super::models::*;
use crate::error::AppResult;

pub async fn list_categories(pool: &PgPool) -> AppResult<Vec<ForumCategory>> {
    let categories = sqlx::query_as::<_, ForumCategory>(
        "SELECT * FROM forum_categories ORDER BY sort_order ASC",
    )
    .fetch_all(pool)
    .await?;

    Ok(categories)
}

pub async fn create_thread(
    pool: &PgPool,
    category_id: Uuid,
    user_id: Uuid,
    title: &str,
    slug: &str,
    body: &str,
) -> AppResult<ForumThread> {
    let thread = sqlx::query_as::<_, ForumThread>(
        "INSERT INTO forum_threads (category_id, user_id, title, slug, body) VALUES ($1, $2, $3, $4, $5) RETURNING *"
    )
    .bind(category_id)
    .bind(user_id)
    .bind(title)
    .bind(slug)
    .bind(body)
    .fetch_one(pool)
    .await?;

    Ok(thread)
}

pub async fn list_threads_by_category(
    pool: &PgPool,
    category_id: Uuid,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<ForumThread>> {
    let threads = sqlx::query_as::<_, ForumThread>(
        "SELECT * FROM forum_threads WHERE category_id = $1 AND is_visible = TRUE ORDER BY is_pinned DESC, last_reply_at DESC NULLS LAST, created_at DESC LIMIT $2 OFFSET $3"
    )
    .bind(category_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(threads)
}

pub async fn find_thread_by_id(pool: &PgPool, id: Uuid) -> AppResult<Option<ForumThread>> {
    let thread = sqlx::query_as::<_, ForumThread>(
        "SELECT * FROM forum_threads WHERE id = $1 AND is_visible = TRUE",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(thread)
}

pub async fn create_reply(
    pool: &PgPool,
    thread_id: Uuid,
    user_id: Uuid,
    body: &str,
) -> AppResult<ForumReply> {
    let reply = sqlx::query_as::<_, ForumReply>(
        "INSERT INTO forum_replies (thread_id, user_id, body) VALUES ($1, $2, $3) RETURNING *",
    )
    .bind(thread_id)
    .bind(user_id)
    .bind(body)
    .fetch_one(pool)
    .await?;

    Ok(reply)
}

pub async fn list_replies(
    pool: &PgPool,
    thread_id: Uuid,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<ForumReply>> {
    let replies = sqlx::query_as::<_, ForumReply>(
        "SELECT * FROM forum_replies WHERE thread_id = $1 AND is_visible = TRUE ORDER BY created_at ASC LIMIT $2 OFFSET $3"
    )
    .bind(thread_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(replies)
}

pub async fn increment_thread_view(pool: &PgPool, id: Uuid) -> AppResult<()> {
    sqlx::query("UPDATE forum_threads SET view_count = view_count + 1 WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
