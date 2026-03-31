use ab_backend::{routes, state};
use http::header;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use futureauth::{FutureAuth, FutureAuthConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    eprintln!("ab-backend starting...");

    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .idle_timeout(std::time::Duration::from_secs(600))
        .max_lifetime(std::time::Duration::from_secs(1800))
        .connect_lazy(&database_url)?;

    tracing::info!("Database pool created (lazy)");

    let auth = FutureAuth::new(db.clone(), FutureAuthConfig {
        api_url: std::env::var("FUTUREAUTH_API_URL")
            .unwrap_or_else(|_| "https://future-auth.com".to_string()),
        secret_key: std::env::var("FUTUREAUTH_SECRET_KEY")
            .expect("FUTUREAUTH_SECRET_KEY must be set"),
        project_name: "AgentBlogs".to_string(),
        ..Default::default()
    });

    let app_state = state::AppState::new(db.clone(), auth);

    let frontend_url = std::env::var("FRONTEND_URL").unwrap_or_default();
    let cors = if frontend_url.is_empty() {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        CorsLayer::new()
            .allow_origin(frontend_url.parse::<http::HeaderValue>().unwrap())
            .allow_methods([
                http::Method::GET,
                http::Method::POST,
                http::Method::PUT,
                http::Method::PATCH,
                http::Method::DELETE,
                http::Method::OPTIONS,
            ])
            .allow_headers([
                header::AUTHORIZATION,
                header::CONTENT_TYPE,
                header::ACCEPT,
            ])
            .allow_credentials(true)
    };

    let mut app = routes::build_router(app_state.clone()).layer(cors);

    if let Ok(frontend_dir) = std::env::var("FRONTEND_DIR") {
        use tower_http::services::{ServeDir, ServeFile};
        let index = format!("{}/index.html", frontend_dir);
        let serve = ServeDir::new(&frontend_dir).fallback(ServeFile::new(&index));
        app = app.fallback_service(serve);
        tracing::info!("Serving frontend from {}", frontend_dir);
    }

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
