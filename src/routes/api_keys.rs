use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::crypto::{generate_api_key, sha256_hash};
use crate::db::ApiKey;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/{pid}/api-keys", get(list_keys).post(create_key))
        .route("/{pid}/api-keys/{id}", axum::routing::delete(revoke_key))
}

#[derive(Serialize)]
struct ApiKeyResponse {
    id: Uuid,
    name: String,
    key_prefix: String,
    scopes: Vec<String>,
    last_used_at: Option<DateTime<Utc>>,
    expires_at: Option<DateTime<Utc>>,
    revoked: bool,
    created_at: DateTime<Utc>,
}

impl From<ApiKey> for ApiKeyResponse {
    fn from(k: ApiKey) -> Self {
        Self {
            id: k.id,
            name: k.name,
            key_prefix: k.key_prefix,
            scopes: k.scopes,
            last_used_at: k.last_used_at,
            expires_at: k.expires_at,
            revoked: k.revoked,
            created_at: k.created_at,
        }
    }
}

async fn list_keys(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(pid): Path<Uuid>,
) -> Result<Json<Vec<ApiKeyResponse>>, (StatusCode, Json<serde_json::Value>)> {
    crate::auth::verify_project_ownership(&state.db, pid, auth.user_id).await?;

    let keys = sqlx::query_as::<_, ApiKey>(
        "SELECT * FROM api_keys WHERE project_id = $1 ORDER BY created_at DESC",
    )
    .bind(pid)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    Ok(Json(keys.into_iter().map(ApiKeyResponse::from).collect()))
}

#[derive(Deserialize)]
struct CreateKeyRequest {
    name: String,
    scopes: Option<Vec<String>>,
}

#[derive(Serialize)]
struct CreateKeyResponse {
    #[serde(flatten)]
    key: ApiKeyResponse,
    secret: String,
}

async fn create_key(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(pid): Path<Uuid>,
    Json(body): Json<CreateKeyRequest>,
) -> Result<(StatusCode, Json<CreateKeyResponse>), (StatusCode, Json<serde_json::Value>)> {
    crate::auth::verify_project_ownership(&state.db, pid, auth.user_id).await?;

    let raw_key = generate_api_key();
    let key_hash = sha256_hash(&raw_key);
    let key_prefix = &raw_key[..16];
    let scopes = body.scopes.unwrap_or_else(|| vec!["posts:read".to_string()]);

    let key = sqlx::query_as::<_, ApiKey>(
        "INSERT INTO api_keys (project_id, name, key_hash, key_prefix, scopes) VALUES ($1, $2, $3, $4, $5) RETURNING *",
    )
    .bind(pid)
    .bind(&body.name)
    .bind(&key_hash)
    .bind(key_prefix)
    .bind(&scopes)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    Ok((
        StatusCode::CREATED,
        Json(CreateKeyResponse {
            key: ApiKeyResponse::from(key),
            secret: raw_key,
        }),
    ))
}

async fn revoke_key(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((pid, id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    crate::auth::verify_project_ownership(&state.db, pid, auth.user_id).await?;

    sqlx::query("UPDATE api_keys SET revoked = true, revoked_at = now() WHERE id = $1 AND project_id = $2")
        .bind(id)
        .bind(pid)
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
