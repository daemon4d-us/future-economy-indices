// Database management commands

use anyhow::Result;
use database::{init_pool, run_migrations};
use tracing::info;

/// Initialize database and run migrations
pub async fn init_database() -> Result<()> {
    info!("Initializing database");

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/future_economy_indices".to_string());

    println!("\n[DB] Initializing Database");
    println!("   URL: {}", database_url);

    // Initialize connection pool
    println!("\n[+] Connecting to database...");
    let pool = init_pool(&database_url).await?;
    println!("   [+] Connection established");

    // Run migrations
    println!("\n[+] Running migrations...");
    run_migrations(&pool).await?;
    println!("   [+] Migrations complete");

    println!("\n[+] Database initialization complete!");

    Ok(())
}

/// Check database status
pub async fn check_status() -> Result<()> {
    info!("Checking database status");

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/future_economy_indices".to_string());

    println!("\n[DB] Database Status");
    println!("   URL: {}", database_url);

    // Try to connect
    println!("\n[+] Testing connection...");
    match init_pool(&database_url).await {
        Ok(_pool) => {
            println!("   [+] Database is accessible");

            // TODO: Check table counts, last update time, etc.
            println!("\n[+] Statistics:");
            println!("   [!] Detailed statistics require SQL queries");
        }
        Err(e) => {
            println!("   [!] Cannot connect to database");
            println!("   Error: {}", e);
            println!("\n[!] Try running: cargo run --bin cli db init");
        }
    }

    Ok(())
}

/// Reset database (deletes all data)
pub async fn reset_database() -> Result<()> {
    info!("Resetting database");

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/future_economy_indices".to_string());

    println!("\n[!] RESETTING DATABASE");
    println!("   This will DELETE ALL DATA!");
    println!("   URL: {}", database_url);

    let pool = init_pool(&database_url).await?;

    // Drop all tables
    println!("\n[+] Dropping all tables...");
    sqlx::query("DROP TABLE IF EXISTS index_performance CASCADE")
        .execute(&pool)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS index_compositions CASCADE")
        .execute(&pool)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS fundamentals CASCADE")
        .execute(&pool)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS companies CASCADE")
        .execute(&pool)
        .await?;

    println!("   [+] Tables dropped");

    // Re-run migrations
    println!("\n[+] Re-running migrations...");
    run_migrations(&pool).await?;
    println!("   [+] Migrations complete");

    println!("\n[+] Database reset complete!");

    Ok(())
}
