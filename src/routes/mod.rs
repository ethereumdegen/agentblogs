mod admin;
mod api_keys;
mod blog_posts;
mod projects;

use axum::{Json, Router, routing::get};
use serde_json::json;

use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    let internal = Router::new()
        .nest("/projects", projects::router())
        .nest("/projects", api_keys::router())
        .nest("/projects", blog_posts::internal_router())
        .nest("/admin", admin::router());

    let public = blog_posts::public_router();

    Router::new()
        .nest("/api/internal", internal)
        .nest("/api/v1", public)
        .merge(futureauth::axum::auth_router(state.auth.clone()))
        .route("/api/health", get(health))
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}
