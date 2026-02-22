use axum::{
    routing::{get, put},
    Router,
};

use crate::app_state::AppState;

use super::handlers;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list_notifications))
        .route("/unread-count", get(handlers::unread_count))
        .route("/{notification_id}/read", put(handlers::mark_as_read))
        .route("/read-all", put(handlers::mark_all_as_read))
}
