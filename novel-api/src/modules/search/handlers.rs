use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;
use crate::error::{AppError, AppResult};

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub index: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub hits: Vec<serde_json::Value>,
    pub total_hits: Option<usize>,
    pub page: usize,
    pub per_page: usize,
    pub processing_time_ms: usize,
}

pub async fn search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> AppResult<Json<SearchResult>> {
    if query.q.is_empty() {
        return Err(AppError::BadRequest("Query parameter 'q' is required".to_string()));
    }

    let index_name = query.index.as_deref().unwrap_or("novels");
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);

    let index = state.search_client.index(index_name);

    let results = index
        .search()
        .with_query(&query.q)
        .with_limit(per_page)
        .with_offset((page - 1) * per_page)
        .execute::<serde_json::Value>()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Search error: {}", e)))?;

    Ok(Json(SearchResult {
        hits: results.hits.into_iter().map(|h| h.result).collect(),
        total_hits: results.estimated_total_hits,
        page,
        per_page,
        processing_time_ms: results.processing_time_ms,
    }))
}
