use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::auth::AdminAuth;
use crate::db::{PlatformSetting, User};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/status", get(admin_status))
        .route("/stats", get(admin_stats))
        .route("/users", get(list_users))
        .route("/users/{id}", axum::routing::patch(update_user))
        .route(
            "/platforms/{platform}",
            get(get_platform_settings).put(set_platform_setting),
        )
        .route(
            "/platforms/{platform}/{key}",
            axum::routing::delete(delete_platform_setting),
        )
}

async fn admin_status(_auth: AdminAuth) -> Json<serde_json::Value> {
    Json(json!({ "status": "ok", "admin": true }))
}

#[derive(Serialize)]
struct AdminStats {
    total_users: i64,
    total_projects: i64,
    total_posts: i64,
    published_posts: i64,
}

async fn admin_stats(
    State(state): State<AppState>,
    _auth: AdminAuth,
) -> Result<Json<AdminStats>, (StatusCode, Json<serde_json::Value>)> {
    let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);
    let total_projects: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM projects")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);
    let total_posts: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM blog_posts")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);
    let published_posts: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM blog_posts WHERE status = 'published'")
            .fetch_one(&state.db)
            .await
            .unwrap_or(0);

    Ok(Json(AdminStats {
        total_users,
        total_projects,
        total_posts,
        published_posts,
    }))
}

async fn list_users(
    State(state): State<AppState>,
    _auth: AdminAuth,
) -> Result<Json<Vec<User>>, (StatusCode, Json<serde_json::Value>)> {
    let users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC")
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
        })?;

    Ok(Json(users))
}

#[derive(Deserialize)]
struct UpdateUser {
    plan: Option<String>,
    is_admin: Option<bool>,
}

async fn update_user(
    State(state): State<AppState>,
    _auth: AdminAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateUser>,
) -> Result<Json<User>, (StatusCode, Json<serde_json::Value>)> {
    let user = sqlx::query_as::<_, User>(
        "UPDATE users SET plan = COALESCE($1, plan), is_admin = COALESCE($2, is_admin), updated_at = now() WHERE id = $3 RETURNING *",
    )
    .bind(&body.plan)
    .bind(body.is_admin)
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    Ok(Json(user))
}

#[derive(Serialize)]
struct PlatformSettingResponse {
    platform: String,
    key_name: String,
    key_value: String,
    is_secret: bool,
}

async fn get_platform_settings(
    State(state): State<AppState>,
    _auth: AdminAuth,
    Path(platform): Path<String>,
) -> Result<Json<Vec<PlatformSettingResponse>>, (StatusCode, Json<serde_json::Value>)> {
    let settings = sqlx::query_as::<_, PlatformSetting>(
        "SELECT * FROM platform_settings WHERE platform = $1 ORDER BY key_name",
    )
    .bind(&platform)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    Ok(Json(
        settings
            .into_iter()
            .map(|s| PlatformSettingResponse {
                platform: s.platform,
                key_name: s.key_name,
                key_value: if s.is_secret {
                    format!("{}...", &s.key_value[..s.key_value.len().min(8)])
                } else {
                    s.key_value
                },
                is_secret: s.is_secret,
            })
            .collect(),
    ))
}

#[derive(Deserialize)]
struct SetPlatformSetting {
    key_name: String,
    key_value: String,
    is_secret: Option<bool>,
}

async fn set_platform_setting(
    State(state): State<AppState>,
    _auth: AdminAuth,
    Path(platform): Path<String>,
    Json(body): Json<SetPlatformSetting>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let is_secret = body.is_secret.unwrap_or(true);

    sqlx::query(
        r#"INSERT INTO platform_settings (platform, key_name, key_value, is_secret)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (platform, key_name) DO UPDATE SET
               key_value = $3, is_secret = $4, updated_at = now()"#,
    )
    .bind(&platform)
    .bind(&body.key_name)
    .bind(&body.key_value)
    .bind(is_secret)
    .execute(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    Ok(Json(json!({ "ok": true })))
}

async fn delete_platform_setting(
    State(state): State<AppState>,
    _auth: AdminAuth,
    Path((platform, key)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    sqlx::query("DELETE FROM platform_settings WHERE platform = $1 AND key_name = $2")
        .bind(&platform)
        .bind(&key)
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
