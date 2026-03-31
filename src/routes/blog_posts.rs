use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::auth::{ApiKeyAuth, AuthUser};
use crate::db::BlogPost;
use crate::state::AppState;

// ── Internal routes (session-authed) ──

pub fn internal_router() -> Router<AppState> {
    Router::new()
        .route("/{pid}/posts", get(list_posts).post(upsert_post))
        .route(
            "/{pid}/posts/{slug}",
            get(get_post).delete(delete_post),
        )
        .route("/{pid}/posts/{slug}/generate-image", axum::routing::post(generate_image))
}

#[derive(Deserialize)]
struct ListQuery {
    status: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}

async fn list_posts(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(pid): Path<Uuid>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<BlogPost>>, (StatusCode, Json<serde_json::Value>)> {
    crate::auth::verify_project_ownership(&state.db, pid, auth.user_id).await?;

    let limit = q.limit.unwrap_or(50).min(100);
    let offset = q.offset.unwrap_or(0);

    let posts = if let Some(status) = &q.status {
        sqlx::query_as::<_, BlogPost>(
            "SELECT * FROM blog_posts WHERE project_id = $1 AND status = $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4",
        )
        .bind(pid)
        .bind(status)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await
    } else {
        sqlx::query_as::<_, BlogPost>(
            "SELECT * FROM blog_posts WHERE project_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(pid)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await
    }
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    Ok(Json(posts))
}

async fn get_post(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((pid, slug)): Path<(Uuid, String)>,
) -> Result<Json<BlogPost>, (StatusCode, Json<serde_json::Value>)> {
    crate::auth::verify_project_ownership(&state.db, pid, auth.user_id).await?;

    let post = sqlx::query_as::<_, BlogPost>(
        "SELECT * FROM blog_posts WHERE project_id = $1 AND slug = $2",
    )
    .bind(pid)
    .bind(&slug)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Post not found" })),
        )
    })?;

    Ok(Json(post))
}

#[derive(Deserialize)]
struct UpsertPost {
    slug: String,
    title: Option<String>,
    date: Option<String>,
    description: Option<String>,
    author: Option<String>,
    cover: Option<String>,
    markdown: Option<String>,
    status: Option<String>,
    tags: Option<Vec<String>>,
}

async fn upsert_post(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(pid): Path<Uuid>,
    Json(body): Json<UpsertPost>,
) -> Result<(StatusCode, Json<BlogPost>), (StatusCode, Json<serde_json::Value>)> {
    crate::auth::verify_project_ownership(&state.db, pid, auth.user_id).await?;

    let post = sqlx::query_as::<_, BlogPost>(
        r#"INSERT INTO blog_posts (project_id, slug, title, date, description, author, cover, markdown, status, tags)
           VALUES ($1, $2, COALESCE($3, ''), COALESCE($4, ''), COALESCE($5, ''), COALESCE($6, ''), COALESCE($7, ''), COALESCE($8, ''), COALESCE($9, 'draft'), COALESCE($10, '{}'))
           ON CONFLICT (project_id, slug) DO UPDATE SET
               title = COALESCE($3, blog_posts.title),
               date = COALESCE($4, blog_posts.date),
               description = COALESCE($5, blog_posts.description),
               author = COALESCE($6, blog_posts.author),
               cover = COALESCE($7, blog_posts.cover),
               markdown = COALESCE($8, blog_posts.markdown),
               status = COALESCE($9, blog_posts.status),
               tags = COALESCE($10, blog_posts.tags),
               updated_at = now()
           RETURNING *"#,
    )
    .bind(pid)
    .bind(&body.slug)
    .bind(&body.title)
    .bind(&body.date)
    .bind(&body.description)
    .bind(&body.author)
    .bind(&body.cover)
    .bind(&body.markdown)
    .bind(&body.status)
    .bind(&body.tags)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    Ok((StatusCode::CREATED, Json(post)))
}

async fn delete_post(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((pid, slug)): Path<(Uuid, String)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    crate::auth::verify_project_ownership(&state.db, pid, auth.user_id).await?;

    sqlx::query("DELETE FROM blog_posts WHERE project_id = $1 AND slug = $2")
        .bind(pid)
        .bind(&slug)
        .execute(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct GenerateImageRequest {
    prompt: String,
}

async fn generate_image(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((pid, slug)): Path<(Uuid, String)>,
    Json(body): Json<GenerateImageRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    crate::auth::verify_project_ownership(&state.db, pid, auth.user_id).await?;

    if auth.plan != "pro" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Pro plan required for AI image generation" })),
        ));
    }

    // Try fal.ai first
    let fal_key = sqlx::query_scalar::<_, String>(
        "SELECT key_value FROM platform_settings WHERE platform = 'fal' AND key_name = 'api_key'",
    )
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    if let Some(fal_key) = fal_key {
        let resp = state
            .http
            .post("https://fal.run/fal-ai/flux/schnell")
            .header("Authorization", format!("Key {}", fal_key))
            .json(&json!({
                "prompt": body.prompt,
                "image_size": "landscape_16_9",
                "num_images": 1
            }))
            .send()
            .await;

        if let Ok(resp) = resp {
            if resp.status().is_success() {
                if let Ok(data) = resp.json::<serde_json::Value>().await {
                    if let Some(url) = data["images"][0]["url"].as_str() {
                        let url = url.to_string();
                        let _ = sqlx::query(
                            "UPDATE blog_posts SET cover = $1, updated_at = now() WHERE project_id = $2 AND slug = $3",
                        )
                        .bind(&url)
                        .bind(pid)
                        .bind(&slug)
                        .execute(&state.db)
                        .await;

                        return Ok(Json(json!({ "url": url })));
                    }
                }
            }
        }
    }

    // Fallback to OpenAI
    let openai_key = sqlx::query_scalar::<_, String>(
        "SELECT key_value FROM platform_settings WHERE platform = 'openai' AND key_name = 'api_key'",
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "error": "No image generation API configured" })),
        )
    })?;

    let resp = state
        .http
        .post("https://api.openai.com/v1/images/generations")
        .header("Authorization", format!("Bearer {}", openai_key))
        .json(&json!({
            "model": "dall-e-3",
            "prompt": body.prompt,
            "n": 1,
            "size": "1792x1024"
        }))
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "error": e.to_string() })),
            )
        })?;

    let data: serde_json::Value = resp.json().await.map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    let url = data["data"][0]["url"]
        .as_str()
        .ok_or_else(|| {
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "error": "No image URL in response" })),
            )
        })?
        .to_string();

    let _ = sqlx::query(
        "UPDATE blog_posts SET cover = $1, updated_at = now() WHERE project_id = $2 AND slug = $3",
    )
    .bind(&url)
    .bind(pid)
    .bind(&slug)
    .execute(&state.db)
    .await;

    Ok(Json(json!({ "url": url })))
}

// ── Public API routes (API key authed) ──

pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/posts", get(public_list_posts).post(public_create_post))
        .route(
            "/posts/{slug}",
            get(public_get_post).delete(public_delete_post),
        )
        .route("/stats", get(public_stats))
}

#[derive(Deserialize)]
struct PublicListQuery {
    limit: Option<i64>,
    cursor: Option<String>,
    status: Option<String>,
}

#[derive(Serialize)]
struct PublicListResponse {
    posts: Vec<BlogPost>,
    next_cursor: Option<String>,
}

async fn public_list_posts(
    State(state): State<AppState>,
    auth: ApiKeyAuth,
    Query(q): Query<PublicListQuery>,
) -> Result<Json<PublicListResponse>, (StatusCode, Json<serde_json::Value>)> {
    let limit = q.limit.unwrap_or(20).min(100);
    let status = q.status.unwrap_or_else(|| "published".to_string());

    let posts = if let Some(cursor) = &q.cursor {
        let cursor_id: Uuid = cursor.parse().map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid cursor" })),
            )
        })?;
        sqlx::query_as::<_, BlogPost>(
            "SELECT * FROM blog_posts WHERE project_id = $1 AND status = $2 AND id < $3 ORDER BY id DESC LIMIT $4",
        )
        .bind(auth.project_id)
        .bind(&status)
        .bind(cursor_id)
        .bind(limit + 1)
        .fetch_all(&state.db)
        .await
    } else {
        sqlx::query_as::<_, BlogPost>(
            "SELECT * FROM blog_posts WHERE project_id = $1 AND status = $2 ORDER BY id DESC LIMIT $3",
        )
        .bind(auth.project_id)
        .bind(&status)
        .bind(limit + 1)
        .fetch_all(&state.db)
        .await
    }
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    let has_more = posts.len() as i64 > limit;
    let posts: Vec<BlogPost> = posts.into_iter().take(limit as usize).collect();
    let next_cursor = if has_more {
        posts.last().map(|p| p.id.to_string())
    } else {
        None
    };

    Ok(Json(PublicListResponse { posts, next_cursor }))
}

async fn public_get_post(
    State(state): State<AppState>,
    auth: ApiKeyAuth,
    Path(slug): Path<String>,
) -> Result<Json<BlogPost>, (StatusCode, Json<serde_json::Value>)> {
    let post = sqlx::query_as::<_, BlogPost>(
        "SELECT * FROM blog_posts WHERE project_id = $1 AND slug = $2",
    )
    .bind(auth.project_id)
    .bind(&slug)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Post not found" })),
        )
    })?;

    Ok(Json(post))
}

async fn public_create_post(
    State(state): State<AppState>,
    auth: ApiKeyAuth,
    Json(body): Json<UpsertPost>,
) -> Result<(StatusCode, Json<BlogPost>), (StatusCode, Json<serde_json::Value>)> {
    if !auth.scopes.iter().any(|s| s == "posts:write" || s == "*") {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Insufficient scope. Required: posts:write" })),
        ));
    }

    let post = sqlx::query_as::<_, BlogPost>(
        r#"INSERT INTO blog_posts (project_id, slug, title, date, description, author, cover, markdown, status, tags)
           VALUES ($1, $2, COALESCE($3, ''), COALESCE($4, ''), COALESCE($5, ''), COALESCE($6, ''), COALESCE($7, ''), COALESCE($8, ''), COALESCE($9, 'draft'), COALESCE($10, '{}'))
           ON CONFLICT (project_id, slug) DO UPDATE SET
               title = COALESCE($3, blog_posts.title),
               date = COALESCE($4, blog_posts.date),
               description = COALESCE($5, blog_posts.description),
               author = COALESCE($6, blog_posts.author),
               cover = COALESCE($7, blog_posts.cover),
               markdown = COALESCE($8, blog_posts.markdown),
               status = COALESCE($9, blog_posts.status),
               tags = COALESCE($10, blog_posts.tags),
               updated_at = now()
           RETURNING *"#,
    )
    .bind(auth.project_id)
    .bind(&body.slug)
    .bind(&body.title)
    .bind(&body.date)
    .bind(&body.description)
    .bind(&body.author)
    .bind(&body.cover)
    .bind(&body.markdown)
    .bind(&body.status)
    .bind(&body.tags)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    Ok((StatusCode::CREATED, Json(post)))
}

async fn public_delete_post(
    State(state): State<AppState>,
    auth: ApiKeyAuth,
    Path(slug): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    if !auth.scopes.iter().any(|s| s == "posts:write" || s == "*") {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Insufficient scope. Required: posts:write" })),
        ));
    }

    sqlx::query("DELETE FROM blog_posts WHERE project_id = $1 AND slug = $2")
        .bind(auth.project_id)
        .bind(&slug)
        .execute(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
struct StatsResponse {
    total_posts: i64,
    published_posts: i64,
    draft_posts: i64,
}

async fn public_stats(
    State(state): State<AppState>,
    auth: ApiKeyAuth,
) -> Result<Json<StatsResponse>, (StatusCode, Json<serde_json::Value>)> {
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM blog_posts WHERE project_id = $1")
        .bind(auth.project_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
        })?;

    let published: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM blog_posts WHERE project_id = $1 AND status = 'published'",
    )
    .bind(auth.project_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    Ok(Json(StatsResponse {
        total_posts: total,
        published_posts: published,
        draft_posts: total - published,
    }))
}
