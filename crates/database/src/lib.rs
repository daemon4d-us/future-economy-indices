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
    // Index composition queries
    get_current_index_composition,
    // Fundamental queries
    get_fundamentals_by_company,
    get_index_composition_as_of,
    get_index_composition_with_companies,
    // Metadata queries
    get_index_metadata,
    // Index performance queries
    get_index_performance,
    get_index_rebalance_dates,
    get_latest_fundamental,
    get_latest_index_performance,
    insert_fundamental,
    insert_index_composition,
    insert_index_performance,
    upsert_company,
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
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}
