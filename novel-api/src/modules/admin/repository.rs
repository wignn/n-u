use sqlx::PgPool;
use uuid::Uuid;

use super::models::*;
use crate::error::{AppError, AppResult};

pub async fn create_report(
    pool: &PgPool,
    reporter_id: Uuid,
    entity_type: &str,
    entity_id: Uuid,
    reason: &str,
) -> AppResult<Report> {
    let report = sqlx::query_as::<_, Report>(
        "INSERT INTO reports (reporter_id, entity_type, entity_id, reason) VALUES ($1, $2::content_type, $3, $4) RETURNING *"
    )
    .bind(reporter_id)
    .bind(entity_type)
    .bind(entity_id)
    .bind(reason)
    .fetch_one(pool)
    .await?;

    Ok(report)
}

pub async fn list_reports(
    pool: &PgPool,
    status: Option<&str>,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<Report>> {
    let reports = if let Some(s) = status {
        sqlx::query_as::<_, Report>(
            "SELECT * FROM reports WHERE status = $1::report_status ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(s)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, Report>(
            "SELECT * FROM reports ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    };

    Ok(reports)
}

pub async fn resolve_report(
    pool: &PgPool,
    report_id: Uuid,
    moderator_id: Uuid,
    status: &str,
    note: Option<&str>,
) -> AppResult<Report> {
    let report = sqlx::query_as::<_, Report>(
        "UPDATE reports SET status = $2::report_status, moderator_id = $3, moderator_note = $4, resolved_at = NOW() WHERE id = $1 RETURNING *"
    )
    .bind(report_id)
    .bind(status)
    .bind(moderator_id)
    .bind(note)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Report not found".to_string()))?;

    Ok(report)
}

pub async fn set_shadowban(pool: &PgPool, user_id: Uuid, shadowban: bool) -> AppResult<()> {
    let result = sqlx::query("UPDATE users SET is_shadowbanned = $2 WHERE id = $1")
        .bind(user_id)
        .bind(shadowban)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("User not found".to_string()));
    }
    Ok(())
}

pub async fn update_user_status(pool: &PgPool, user_id: Uuid, status: &str) -> AppResult<()> {
    let result = sqlx::query("UPDATE users SET status = $2::user_status WHERE id = $1")
        .bind(user_id)
        .bind(status)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("User not found".to_string()));
    }
    Ok(())
}

pub async fn get_stats(pool: &PgPool) -> AppResult<AdminStats> {
    let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;

    let total_novels: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM novels")
        .fetch_one(pool)
        .await?;

    let total_reviews: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM reviews")
        .fetch_one(pool)
        .await?;

    let total_reports_pending: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM reports WHERE status = 'pending'")
            .fetch_one(pool)
            .await?;

    let total_forum_threads: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM forum_threads")
        .fetch_one(pool)
        .await?;

    Ok(AdminStats {
        total_users,
        total_novels,
        total_reviews,
        total_reports_pending,
        total_forum_threads,
    })
}
