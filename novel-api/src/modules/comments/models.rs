use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Comment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub depth: i16,
    pub body: String,
    pub upvote_count: i32,
    pub is_visible: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct CommentWithAuthor {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub avatar_url: Option<String>,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub depth: i16,
    pub body: String,
    pub upvote_count: i32,
    pub created_at: DateTime<Utc>,
    pub children: Vec<CommentWithAuthor>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCommentRequest {
    pub entity_type: String,
    pub entity_id: Uuid,
    pub parent_id: Option<Uuid>,
    #[validate(length(min = 1, max = 5000))]
    pub body: String,
}
