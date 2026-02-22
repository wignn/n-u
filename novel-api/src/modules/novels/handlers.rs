use axum::extract::{Path, Query, State};
use axum::Json;
use validator::Validate;

use crate::app_state::AppState;
use crate::error::{AppError, AppResult};
use crate::events::outbox::insert_outbox_event;
use crate::middleware::auth::AuthUser;

use super::models::*;
use super::repository;

pub async fn create_novel(
    State(state): State<AppState>,
    axum::extract::Extension(auth_user): axum::extract::Extension<AuthUser>,
    Json(req): Json<CreateNovelRequest>,
) -> AppResult<Json<Novel>> {
    req.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let novel_slug = slug::slugify(&req.title);

    let mut tx = state.db.begin().await?;

    let novel = sqlx::query_as::<_, Novel>(
        "INSERT INTO novels (author_id, title, slug, synopsis, cover_url, genres, tags) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *"
    )
    .bind(auth_user.user_id)
    .bind(&req.title)
    .bind(&novel_slug)
    .bind(req.synopsis.as_deref())
    .bind(req.cover_url.as_deref())
    .bind(&req.genres.unwrap_or_default())
    .bind(&req.tags.unwrap_or_default())
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref db_err) if db_err.constraint().is_some() => {
            AppError::Conflict("Novel with this slug already exists".to_string())
        }
        _ => AppError::Database(e),
    })?;

    insert_outbox_event(&mut tx, novel.id, "novel.novel.created").await?;

    tx.commit().await?;

    Ok(Json(novel))
}

pub async fn get_novel(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> AppResult<Json<Novel>> {
    let novel = repository::find_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Novel not found".to_string()))?;

    repository::increment_view(&state.db, id).await.ok();

    Ok(Json(novel))
}

pub async fn list_novels(
    State(state): State<AppState>,
    Query(query): Query<NovelListQuery>,
) -> AppResult<Json<PaginatedResponse<Novel>>> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);

    let (novels, total) = repository::list(
        &state.db,
        page,
        per_page,
        query.status.as_deref(),
        query.genre.as_deref(),
        query.sort.as_deref(),
    )
    .await?;

    Ok(Json(PaginatedResponse {
        data: novels,
        total,
        page,
        per_page,
    }))
}

pub async fn update_novel(
    State(state): State<AppState>,
    axum::extract::Extension(auth_user): axum::extract::Extension<AuthUser>,
    Path(id): Path<uuid::Uuid>,
    Json(req): Json<UpdateNovelRequest>,
) -> AppResult<Json<Novel>> {
    req.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let mut tx = state.db.begin().await?;

    let novel = sqlx::query_as::<_, Novel>(
        "UPDATE novels SET title = COALESCE($3, title), synopsis = COALESCE($4, synopsis), cover_url = COALESCE($5, cover_url), status = COALESCE($6, status), genres = COALESCE($7, genres), tags = COALESCE($8, tags) WHERE id = $1 AND author_id = $2 RETURNING *"
    )
    .bind(id)
    .bind(auth_user.user_id)
    .bind(req.title.as_deref())
    .bind(req.synopsis.as_deref())
    .bind(req.cover_url.as_deref())
    .bind(req.status.as_deref())
    .bind(req.genres.as_deref())
    .bind(req.tags.as_deref())
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Novel not found or unauthorized".to_string()))?;

    insert_outbox_event(&mut tx, novel.id, "novel.novel.updated").await?;

    tx.commit().await?;

    Ok(Json(novel))
}

pub async fn delete_novel(
    State(state): State<AppState>,
    axum::extract::Extension(auth_user): axum::extract::Extension<AuthUser>,
    Path(id): Path<uuid::Uuid>,
) -> AppResult<axum::http::StatusCode> {
    let mut tx = state.db.begin().await?;

    let result = sqlx::query("DELETE FROM novels WHERE id = $1 AND author_id = $2")
        .bind(id)
        .bind(auth_user.user_id)
        .execute(&mut *tx)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Novel not found or unauthorized".to_string()));
    }

    insert_outbox_event(&mut tx, id, "novel.novel.deleted").await?;

    tx.commit().await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub async fn get_novel_by_slug(
    State(state): State<AppState>,
    Path(slug_param): Path<String>,
) -> AppResult<Json<Novel>> {
    let novel = repository::find_by_slug(&state.db, &slug_param)
        .await?
        .ok_or_else(|| AppError::NotFound("Novel not found".to_string()))?;

    repository::increment_view(&state.db, novel.id).await.ok();

    Ok(Json(novel))
}
