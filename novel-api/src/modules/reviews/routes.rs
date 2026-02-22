use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::app_state::AppState;

use super::handlers;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/novels/{novel_id}/reviews", get(handlers::list_reviews))
        .route("/novels/{novel_id}/reviews", post(handlers::create_review))
        .route(
            "/novels/{novel_id}/reviews/{review_id}",
            put(handlers::update_review),
        )
        .route(
            "/novels/{novel_id}/reviews/{review_id}",
            delete(handlers::delete_review),
        )
}
