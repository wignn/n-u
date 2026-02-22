use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ForumCategory {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub is_locked: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ForumThread {
    pub id: Uuid,
    pub category_id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub slug: String,
    pub body: String,
    pub is_pinned: bool,
    pub is_locked: bool,
    pub is_visible: bool,
    pub reply_count: i32,
    pub last_reply_at: Option<DateTime<Utc>>,
    pub view_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ForumReply {
    pub id: Uuid,
    pub thread_id: Uuid,
    pub user_id: Uuid,
    pub body: String,
    pub is_visible: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateThreadRequest {
    pub category_id: Uuid,
    #[validate(length(min = 1, max = 500))]
    pub title: String,
    #[validate(length(min = 1, max = 50000))]
    pub body: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateReplyRequest {
    #[validate(length(min = 1, max = 50000))]
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct ThreadListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}
