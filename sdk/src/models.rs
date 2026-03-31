use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlogPost {
    pub id: Uuid,
    pub project_id: Uuid,
    pub slug: String,
    pub title: String,
    pub date: String,
    pub description: String,
    pub author: String,
    pub cover: String,
    pub markdown: String,
    pub status: String,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostListResponse {
    pub posts: Vec<BlogPost>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct PostQuery {
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CreatePostRequest {
    pub slug: String,
    pub title: Option<String>,
    pub date: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub cover: Option<String>,
    pub markdown: Option<String>,
    pub status: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsResponse {
    pub total_posts: i64,
    pub published_posts: i64,
    pub draft_posts: i64,
}
