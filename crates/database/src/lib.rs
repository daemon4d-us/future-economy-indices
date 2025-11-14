// Database crate - PostgreSQL models and migrations

use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

pub mod models;
pub mod schema;

pub use models::*;
pub use schema::{
    // Company queries
    get_all_companies,
    get_companies_by_space_score,
    get_company_by_ticker,
    upsert_company,
    // Fundamental queries
    get_fundamentals_by_company,
    get_latest_fundamental,
    insert_fundamental,
    // Index composition queries
    get_current_index_composition,
    get_index_composition_as_of,
    get_index_composition_with_companies,
    get_index_rebalance_dates,
    insert_index_composition,
    // Index performance queries
    get_index_performance,
    get_latest_index_performance,
    insert_index_performance,
    // Metadata queries
    get_index_metadata,
    // Types
    CompositionWithCompany,
    IndexMetadata,
};

/// Initialize database connection pool
pub async fn init_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await?;
    Ok(pool)
}

/// Run database migrations
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await?;
    Ok(())
}
