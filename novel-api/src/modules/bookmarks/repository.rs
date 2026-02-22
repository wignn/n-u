use sqlx::PgPool;
use uuid::Uuid;

use super::models::*;
use crate::error::{AppError, AppResult};

pub async fn add_bookmark(pool: &PgPool, user_id: Uuid, novel_id: Uuid) -> AppResult<Bookmark> {
    let bookmark = sqlx::query_as::<_, Bookmark>(
        "INSERT INTO bookmarks (user_id, novel_id) VALUES ($1, $2) ON CONFLICT (user_id, novel_id) DO UPDATE SET created_at = bookmarks.created_at RETURNING *"
    )
    .bind(user_id)
    .bind(novel_id)
    .fetch_one(pool)
    .await?;

    Ok(bookmark)
}

pub async fn remove_bookmark(pool: &PgPool, user_id: Uuid, novel_id: Uuid) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM bookmarks WHERE user_id = $1 AND novel_id = $2")
        .bind(user_id)
        .bind(novel_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Bookmark not found".to_string()));
    }
    Ok(())
}

pub async fn list_bookmarks(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<Bookmark>> {
    let bookmarks = sqlx::query_as::<_, Bookmark>(
        "SELECT * FROM bookmarks WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(bookmarks)
}

pub async fn upsert_reading_history(
    pool: &PgPool,
    user_id: Uuid,
    novel_id: Uuid,
    chapter_id: Uuid,
    chapter_number: i32,
) -> AppResult<ReadingHistory> {
    let history = sqlx::query_as::<_, ReadingHistory>(
        "INSERT INTO reading_history (user_id, novel_id, last_chapter_id, last_chapter_number) VALUES ($1, $2, $3, $4) ON CONFLICT (user_id, novel_id) DO UPDATE SET last_chapter_id = $3, last_chapter_number = $4, updated_at = NOW() RETURNING *"
    )
    .bind(user_id)
    .bind(novel_id)
    .bind(chapter_id)
    .bind(chapter_number)
    .fetch_one(pool)
    .await?;

    Ok(history)
}

pub async fn get_reading_history(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<ReadingHistory>> {
    let history = sqlx::query_as::<_, ReadingHistory>(
        "SELECT * FROM reading_history WHERE user_id = $1 ORDER BY updated_at DESC LIMIT $2 OFFSET $3"
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(history)
}
