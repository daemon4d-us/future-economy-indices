// Application state

use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<PgPool>,
}

impl AppState {
    pub fn new(db: PgPool) -> Self {
        Self { db: Arc::new(db) }
    }
}
