use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Report {
    pub id: Uuid,
    pub reporter_id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub reason: String,
    pub status: String,
    pub moderator_id: Option<Uuid>,
    pub moderator_note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateReportRequest {
    pub entity_type: String,
    pub entity_id: Uuid,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct ResolveReportRequest {
    pub status: String,
    pub moderator_note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ShadowbanRequest {
    pub user_id: Uuid,
    pub shadowban: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserStatusRequest {
    pub user_id: Uuid,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct ReportListQuery {
    pub status: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct AdminStats {
    pub total_users: i64,
    pub total_novels: i64,
    pub total_reviews: i64,
    pub total_reports_pending: i64,
    pub total_forum_threads: i64,
}
