use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use tracing::warn;
use uuid::Uuid;

use crate::crypto::sha256_hash;
use crate::state::AppState;

#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidToken(String),
    UserNotFound,
    Forbidden,
    InternalError(String),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "Missing authorization token"),
            AuthError::InvalidToken(_) => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AuthError::UserNotFound => (StatusCode::UNAUTHORIZED, "User not found"),
            AuthError::Forbidden => (StatusCode::FORBIDDEN, "Admin access required"),
            AuthError::InternalError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };
        (status, axum::Json(serde_json::json!({ "error": message }))).into_response()
    }
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub futureauth_user_id: String,
    pub user_id: Uuid,
    pub plan: String,
    pub is_admin: bool,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let cookie_header = parts
            .headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let session_token = cookie_header
            .split(';')
            .filter_map(|c| {
                let c = c.trim();
                c.strip_prefix("futureauth_session=")
            })
            .next()
            .ok_or(AuthError::MissingToken)?;

        let session = sqlx::query_as::<_, (String,)>(
            r#"SELECT user_id FROM session WHERE token = $1 AND expires_at > NOW()"#,
        )
        .bind(session_token)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| {
            warn!("Session query failed: {e}");
            AuthError::InternalError(e.to_string())
        })?
        .ok_or(AuthError::MissingToken)?;

        let futureauth_user_id = &session.0;

        let fa_user = sqlx::query_as::<_, (Option<String>, Option<String>)>(
            r#"SELECT email, name FROM "user" WHERE id = $1"#,
        )
        .bind(futureauth_user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| {
            warn!("FutureAuth user query failed: {e}");
            AuthError::InternalError(e.to_string())
        })?
        .ok_or(AuthError::UserNotFound)?;

        let email = fa_user.0.unwrap_or_default();
        let name = fa_user.1.unwrap_or_default();

        let existing = sqlx::query_as::<_, (Uuid, String, bool)>(
            "SELECT id, plan, is_admin FROM users WHERE futureauth_user_id = $1",
        )
        .bind(futureauth_user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| {
            warn!("User lookup failed: {e}");
            AuthError::InternalError(e.to_string())
        })?;

        let (user_id, plan, is_admin) = if let Some(row) = existing {
            row
        } else {
            sqlx::query_as::<_, (Uuid, String, bool)>(
                "INSERT INTO users (futureauth_user_id, email, name)
                 VALUES ($1, $2, $3)
                 RETURNING id, plan, is_admin",
            )
            .bind(futureauth_user_id)
            .bind(&email)
            .bind(&name)
            .fetch_one(&state.db)
            .await
            .map_err(|e| {
                warn!("Failed to create user: {e}");
                AuthError::InternalError(e.to_string())
            })?
        };

        Ok(AuthUser {
            futureauth_user_id: futureauth_user_id.clone(),
            user_id,
            plan,
            is_admin,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ApiKeyAuth {
    pub project_id: Uuid,
    pub scopes: Vec<String>,
    pub key_id: Uuid,
}

impl FromRequestParts<AppState> for ApiKeyAuth {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = extract_bearer_token(parts).ok_or(AuthError::MissingToken)?;

        if !token.starts_with("ab_live_") {
            return Err(AuthError::InvalidToken(
                "Invalid API key format".to_string(),
            ));
        }

        let key_hash = sha256_hash(&token);

        let row = sqlx::query_as::<_, (Uuid, Uuid, Vec<String>)>(
            "SELECT id, project_id, scopes FROM api_keys WHERE key_hash = $1 AND revoked = false AND (expires_at IS NULL OR expires_at > now())",
        )
        .bind(&key_hash)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| AuthError::InternalError(e.to_string()))?
        .ok_or(AuthError::InvalidToken("API key not found or revoked".to_string()))?;

        let _ = sqlx::query("UPDATE api_keys SET last_used_at = now() WHERE id = $1")
            .bind(row.0)
            .execute(&state.db)
            .await;

        Ok(ApiKeyAuth {
            key_id: row.0,
            project_id: row.1,
            scopes: row.2,
        })
    }
}

pub async fn verify_project_ownership(
    db: &sqlx::PgPool,
    project_id: Uuid,
    user_id: Uuid,
) -> Result<(), (StatusCode, axum::Json<serde_json::Value>)> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM projects WHERE id = $1 AND user_id = $2)",
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_one(db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    if !exists {
        return Err((
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": "Project not found" })),
        ));
    }
    Ok(())
}

fn extract_bearer_token(parts: &Parts) -> Option<String> {
    let auth_header = parts.headers.get("authorization")?.to_str().ok()?;
    if let Some(token) = auth_header.strip_prefix("Bearer ") {
        Some(token.to_string())
    } else {
        None
    }
}

#[derive(Debug, Clone)]
pub struct AdminAuth {
    pub futureauth_user_id: String,
    pub user_id: Uuid,
}

impl FromRequestParts<AppState> for AdminAuth {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth = AuthUser::from_request_parts(parts, state).await?;

        if !auth.is_admin {
            return Err(AuthError::Forbidden);
        }

        Ok(AdminAuth {
            futureauth_user_id: auth.futureauth_user_id,
            user_id: auth.user_id,
        })
    }
}
