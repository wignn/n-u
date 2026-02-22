use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Bookmark {
    pub id: Uuid,
    pub user_id: Uuid,
    pub novel_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ReadingHistory {
    pub id: Uuid,
    pub user_id: Uuid,
    pub novel_id: Uuid,
    pub last_chapter_id: Option<Uuid>,
    pub last_chapter_number: i32,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateReadingHistoryRequest {
    pub novel_id: Uuid,
    pub chapter_id: Uuid,
    pub chapter_number: i32,
}
