use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, patch},
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::db::Project;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_projects).post(create_project))
        .route("/{id}", patch(update_project).delete(delete_project))
}

async fn list_projects(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<Project>>, (StatusCode, Json<serde_json::Value>)> {
    let projects = sqlx::query_as::<_, Project>(
        "SELECT * FROM projects WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(auth.user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    Ok(Json(projects))
}

#[derive(Deserialize)]
struct CreateProject {
    name: String,
}

async fn create_project(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateProject>,
) -> Result<(StatusCode, Json<Project>), (StatusCode, Json<serde_json::Value>)> {
    let project = sqlx::query_as::<_, Project>(
        "INSERT INTO projects (user_id, name) VALUES ($1, $2) RETURNING *",
    )
    .bind(auth.user_id)
    .bind(&body.name)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    Ok((StatusCode::CREATED, Json(project)))
}

#[derive(Deserialize)]
struct UpdateProject {
    name: Option<String>,
}

async fn update_project(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateProject>,
) -> Result<Json<Project>, (StatusCode, Json<serde_json::Value>)> {
    crate::auth::verify_project_ownership(&state.db, id, auth.user_id).await?;

    let project = sqlx::query_as::<_, Project>(
        "UPDATE projects SET name = COALESCE($1, name), updated_at = now() WHERE id = $2 RETURNING *",
    )
    .bind(&body.name)
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    Ok(Json(project))
}

async fn delete_project(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    crate::auth::verify_project_ownership(&state.db, id, auth.user_id).await?;

    sqlx::query("DELETE FROM projects WHERE id = $1")
        .bind(id)
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
