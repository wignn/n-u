use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::app_state::AppState;

use super::handlers;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/novels/{novel_id}/chapters", get(handlers::list_chapters))
        .route(
            "/novels/{novel_id}/chapters",
            post(handlers::create_chapter),
        )
        .route(
            "/novels/{novel_id}/chapters/{chapter_id}",
            delete(handlers::delete_chapter),
        )
}
