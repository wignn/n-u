use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Novel {
    pub id: Uuid,
    pub author_id: Uuid,
    pub title: String,
    pub slug: String,
    pub synopsis: Option<String>,
    pub cover_url: Option<String>,
    pub status: String,
    pub genres: Vec<String>,
    pub tags: Vec<String>,
    pub chapter_count: i32,
    pub view_count: i64,
    pub is_visible: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct NovelDetail {
    pub novel: Novel,
    pub author_username: String,
    pub avg_rating: f64,
    pub rating_count: i32,
}

#[derive(Debug, Serialize)]
pub struct NovelListItem {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub cover_url: Option<String>,
    pub status: String,
    pub chapter_count: i32,
    pub author_username: String,
    pub avg_rating: f64,
    pub rating_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateNovelRequest {
    #[validate(length(min = 1, max = 500))]
    pub title: String,
    #[validate(length(max = 10000))]
    pub synopsis: Option<String>,
    pub cover_url: Option<String>,
    pub genres: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateNovelRequest {
    #[validate(length(min = 1, max = 500))]
    pub title: Option<String>,
    #[validate(length(max = 10000))]
    pub synopsis: Option<String>,
    pub cover_url: Option<String>,
    pub status: Option<String>,
    pub genres: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct NovelListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub status: Option<String>,
    pub genre: Option<String>,
    pub sort: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}
