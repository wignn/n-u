use axum::{
    routing::{get, post},
    Router,
};

use crate::app_state::AppState;

use super::handlers;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/categories", get(handlers::list_categories))
        .route("/threads", post(handlers::create_thread))
        .route(
            "/categories/{category_id}/threads",
            get(handlers::list_threads),
        )
        .route("/threads/{thread_id}", get(handlers::get_thread))
        .route("/threads/{thread_id}/replies", get(handlers::list_replies))
        .route("/threads/{thread_id}/replies", post(handlers::create_reply))
}
