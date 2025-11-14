// API server main entry point

use axum::{routing::get, Json, Router};
use serde::Serialize;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod models;
mod routes;
mod state;

use state::AppState;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize database connection pool
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql:///future_economy_indices".to_string());

    tracing::info!("Connecting to database: {}", database_url);
    let pool = database::init_pool(&database_url).await?;
    tracing::info!("Database connection established");

    let state = AppState::new(pool);

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        // Index routes
        .route("/api/indices", get(routes::indices::list_indices))
        .route("/api/indices/:name", get(routes::indices::get_index))
        .route(
            "/api/indices/:name/composition",
            get(routes::indices::get_composition),
        )
        .route(
            "/api/indices/:name/performance",
            get(routes::indices::get_performance),
        )
        .with_state(state)
        .layer(CorsLayer::permissive());

    // Get port from environment or use default
    let port = std::env::var("API_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Starting API server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
