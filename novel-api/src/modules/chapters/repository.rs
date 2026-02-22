use sqlx::PgPool;
use uuid::Uuid;

use super::models::Chapter;
use crate::error::{AppError, AppResult};

pub async fn create(
    pool: &PgPool,
    novel_id: Uuid,
    chapter_number: i32,
    title: Option<&str>,
    link: &str,
) -> AppResult<Chapter> {
    let chapter = sqlx::query_as::<_, Chapter>(
        "INSERT INTO chapters (novel_id, chapter_number, title, link) VALUES ($1, $2, $3, $4) RETURNING *"
    )
    .bind(novel_id)
    .bind(chapter_number)
    .bind(title)
    .bind(link)
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref db_err) if db_err.constraint().is_some() => {
            AppError::Conflict("Chapter number already exists for this novel".to_string())
        }
        _ => AppError::Database(e),
    })?;

    Ok(chapter)
}

pub async fn list_by_novel(pool: &PgPool, novel_id: Uuid) -> AppResult<Vec<Chapter>> {
    let chapters = sqlx::query_as::<_, Chapter>(
        "SELECT * FROM chapters WHERE novel_id = $1 ORDER BY chapter_number ASC"
    )
    .bind(novel_id)
    .fetch_all(pool)
    .await?;

    Ok(chapters)
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> AppResult<Option<Chapter>> {
    let chapter = sqlx::query_as::<_, Chapter>("SELECT * FROM chapters WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(chapter)
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    title: Option<&str>,
    link: Option<&str>,
) -> AppResult<Chapter> {
    let chapter = sqlx::query_as::<_, Chapter>(
        "UPDATE chapters SET title = COALESCE($2, title), link = COALESCE($3, link) WHERE id = $1 RETURNING *"
    )
    .bind(id)
    .bind(title)
    .bind(link)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Chapter not found".to_string()))?;

    Ok(chapter)
}

pub async fn delete(pool: &PgPool, id: Uuid) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM chapters WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Chapter not found".to_string()));
    }
    Ok(())
}
