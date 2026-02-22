use sqlx::PgPool;
use uuid::Uuid;

use super::models::*;
use crate::error::{AppError, AppResult};

pub async fn create(
    pool: &PgPool,
    author_id: Uuid,
    title: &str,
    slug: &str,
    synopsis: Option<&str>,
    cover_url: Option<&str>,
    genres: &[String],
    tags: &[String],
) -> AppResult<Novel> {
    let novel = sqlx::query_as::<_, Novel>(
        "INSERT INTO novels (author_id, title, slug, synopsis, cover_url, genres, tags) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *"
    )
    .bind(author_id)
    .bind(title)
    .bind(slug)
    .bind(synopsis)
    .bind(cover_url)
    .bind(genres)
    .bind(tags)
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref db_err) if db_err.constraint().is_some() => {
            AppError::Conflict("Novel with this title already exists".to_string())
        }
        _ => AppError::Database(e),
    })?;

    Ok(novel)
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> AppResult<Option<Novel>> {
    let novel = sqlx::query_as::<_, Novel>(
        "SELECT * FROM novels WHERE id = $1 AND is_visible = TRUE"
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(novel)
}

pub async fn find_by_slug(pool: &PgPool, slug: &str) -> AppResult<Option<Novel>> {
    let novel = sqlx::query_as::<_, Novel>(
        "SELECT * FROM novels WHERE slug = $1 AND is_visible = TRUE"
    )
    .bind(slug)
    .fetch_optional(pool)
    .await?;
    Ok(novel)
}

pub async fn list(
    pool: &PgPool,
    page: i64,
    per_page: i64,
    status: Option<&str>,
    genre: Option<&str>,
    sort: Option<&str>,
) -> AppResult<(Vec<Novel>, i64)> {
    let offset = (page - 1) * per_page;

    let sort_clause = match sort.unwrap_or("latest") {
        "popular" => "view_count DESC",
        "rating" => "view_count DESC",
        "chapters" => "chapter_count DESC",
        _ => "created_at DESC",
    };

    let query = format!(
        "SELECT * FROM novels WHERE is_visible = TRUE {} {} ORDER BY {} LIMIT $1 OFFSET $2",
        status.map(|_| "AND status = $3").unwrap_or(""),
        genre.map(|_| if status.is_some() { "AND $4 = ANY(genres)" } else { "AND $3 = ANY(genres)" }).unwrap_or(""),
        sort_clause,
    );

    let novels = if let (Some(s), Some(g)) = (status, genre) {
        sqlx::query_as::<_, Novel>(&query)
            .bind(per_page)
            .bind(offset)
            .bind(s)
            .bind(g)
            .fetch_all(pool)
            .await?
    } else if let Some(s) = status {
        sqlx::query_as::<_, Novel>(&query)
            .bind(per_page)
            .bind(offset)
            .bind(s)
            .fetch_all(pool)
            .await?
    } else if let Some(g) = genre {
        sqlx::query_as::<_, Novel>(&query)
            .bind(per_page)
            .bind(offset)
            .bind(g)
            .fetch_all(pool)
            .await?
    } else {
        sqlx::query_as::<_, Novel>(&query)
            .bind(per_page)
            .bind(offset)
            .fetch_all(pool)
            .await?
    };

    let count_query = "SELECT COUNT(*) FROM novels WHERE is_visible = TRUE";
    let total: i64 = sqlx::query_scalar(count_query).fetch_one(pool).await?;

    Ok((novels, total))
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    author_id: Uuid,
    title: Option<&str>,
    synopsis: Option<&str>,
    cover_url: Option<&str>,
    status: Option<&str>,
    genres: Option<&[String]>,
    tags: Option<&[String]>,
) -> AppResult<Novel> {
    let novel = sqlx::query_as::<_, Novel>(
        "UPDATE novels SET title = COALESCE($3, title), synopsis = COALESCE($4, synopsis), cover_url = COALESCE($5, cover_url), status = COALESCE($6, status), genres = COALESCE($7, genres), tags = COALESCE($8, tags), slug = COALESCE($9, slug) WHERE id = $1 AND author_id = $2 RETURNING *"
    )
    .bind(id)
    .bind(author_id)
    .bind(title)
    .bind(synopsis)
    .bind(cover_url)
    .bind(status)
    .bind(genres)
    .bind(tags)
    .bind(title.map(slug::slugify))
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Novel not found or unauthorized".to_string()))?;

    Ok(novel)
}

pub async fn delete(pool: &PgPool, id: Uuid, author_id: Uuid) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM novels WHERE id = $1 AND author_id = $2")
        .bind(id)
        .bind(author_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Novel not found or unauthorized".to_string()));
    }
    Ok(())
}

pub async fn increment_view(pool: &PgPool, id: Uuid) -> AppResult<()> {
    sqlx::query("UPDATE novels SET view_count = view_count + 1 WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
