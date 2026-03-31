use futureauth::FutureAuth;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub http: reqwest::Client,
    pub auth: Arc<FutureAuth>,
}

impl AppState {
    pub fn new(db: PgPool, auth: Arc<FutureAuth>) -> Self {
        Self {
            db,
            http: reqwest::Client::new(),
            auth,
        }
    }
}

impl AsRef<Arc<FutureAuth>> for AppState {
    fn as_ref(&self) -> &Arc<FutureAuth> {
        &self.auth
    }
}
