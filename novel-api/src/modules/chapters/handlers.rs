use axum::extract::{Path, State};
use axum::Json;
use validator::Validate;

use crate::app_state::AppState;
use crate::error::{AppError, AppResult};
use crate::events::outbox::insert_outbox_event;
use crate::middleware::auth::AuthUser;
use crate::modules::novels;

use super::models::*;
use super::repository;

pub async fn create_chapter(
    State(state): State<AppState>,
    axum::extract::Extension(auth_user): axum::extract::Extension<AuthUser>,
    Path(novel_id): Path<uuid::Uuid>,
    Json(req): Json<CreateChapterRequest>,
) -> AppResult<Json<Chapter>> {
    req.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let novel = novels::repository::find_by_id(&state.db, novel_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Novel not found".to_string()))?;

    if novel.author_id != auth_user.user_id {
        return Err(AppError::Forbidden);
    }

    let mut tx = state.db.begin().await?;

    let chapter = sqlx::query_as::<_, Chapter>(
        "INSERT INTO chapters (novel_id, chapter_number, title, link) VALUES ($1, $2, $3, $4) RETURNING *"
    )
    .bind(novel_id)
    .bind(req.chapter_number)
    .bind(req.title.as_deref())
    .bind(&req.link)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref db_err) if db_err.constraint().is_some() => {
            AppError::Conflict("Chapter number already exists".to_string())
        }
        _ => AppError::Database(e),
    })?;

    insert_outbox_event(&mut tx, chapter.id, "novel.chapter.created").await?;

    tx.commit().await?;

    Ok(Json(chapter))
}

pub async fn list_chapters(
    State(state): State<AppState>,
    Path(novel_id): Path<uuid::Uuid>,
) -> AppResult<Json<Vec<Chapter>>> {
    let chapters = repository::list_by_novel(&state.db, novel_id).await?;
    Ok(Json(chapters))
}

pub async fn delete_chapter(
    State(state): State<AppState>,
    axum::extract::Extension(auth_user): axum::extract::Extension<AuthUser>,
    Path((novel_id, chapter_id)): Path<(uuid::Uuid, uuid::Uuid)>,
) -> AppResult<axum::http::StatusCode> {
    let novel = novels::repository::find_by_id(&state.db, novel_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Novel not found".to_string()))?;

    if novel.author_id != auth_user.user_id {
        return Err(AppError::Forbidden);
    }

    let mut tx = state.db.begin().await?;

    let result = sqlx::query("DELETE FROM chapters WHERE id = $1 AND novel_id = $2")
        .bind(chapter_id)
        .bind(novel_id)
        .execute(&mut *tx)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Chapter not found".to_string()));
    }

    insert_outbox_event(&mut tx, chapter_id, "novel.chapter.deleted").await?;

    tx.commit().await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
