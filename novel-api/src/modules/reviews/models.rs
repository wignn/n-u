use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Review {
    pub id: Uuid,
    pub user_id: Uuid,
    pub novel_id: Uuid,
    pub rating: i16,
    pub title: Option<String>,
    pub body: String,
    pub upvote_count: i32,
    pub is_visible: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateReviewRequest {
    #[validate(range(min = 1, max = 5))]
    pub rating: i16,
    #[validate(length(max = 300))]
    pub title: Option<String>,
    #[validate(length(min = 10, max = 10000))]
    pub body: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateReviewRequest {
    #[validate(range(min = 1, max = 5))]
    pub rating: Option<i16>,
    #[validate(length(max = 300))]
    pub title: Option<String>,
    #[validate(length(min = 10, max = 10000))]
    pub body: Option<String>,
}
