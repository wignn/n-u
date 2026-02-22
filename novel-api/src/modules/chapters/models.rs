use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Chapter {
    pub id: Uuid,
    pub novel_id: Uuid,
    pub chapter_number: i32,
    pub title: Option<String>,
    pub link: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateChapterRequest {
    pub chapter_number: i32,
    #[validate(length(max = 500))]
    pub title: Option<String>,
    #[validate(url)]
    pub link: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateChapterRequest {
    #[validate(length(max = 500))]
    pub title: Option<String>,
    #[validate(url)]
    pub link: Option<String>,
}
