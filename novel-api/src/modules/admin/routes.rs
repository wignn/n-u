use axum::{
    routing::{get, post, put},
    Router,
};

use crate::app_state::AppState;

use super::handlers;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/reports", post(handlers::create_report))
        .route("/reports", get(handlers::list_reports))
        .route(
            "/reports/{report_id}/resolve",
            put(handlers::resolve_report),
        )
        .route("/shadowban", put(handlers::set_shadowban))
        .route("/user-status", put(handlers::update_user_status))
        .route("/stats", get(handlers::get_stats))
}
