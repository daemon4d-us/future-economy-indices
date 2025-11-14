// Future Economy Indices CLI
//
// Command-line tool for managing indices, data ingestion, and operations

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod commands;

#[derive(Parser)]
#[command(name = "future-indices-cli")]
#[command(about = "Future Economy Indices - CLI Tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Data management commands
    #[command(subcommand)]
    Data(DataCommands),

    /// Index operation commands
    #[command(subcommand)]
    Index(IndexCommands),

    /// Database management commands
    #[command(subcommand)]
    Db(DbCommands),
}

#[derive(Subcommand)]
enum DataCommands {
    /// Fetch and ingest ticker data from Polygon.io
    Ingest {
        /// Ticker symbol (e.g., RKLB)
        #[arg(short, long)]
        ticker: String,
    },

    /// Classify a company using AI
    Classify {
        /// Ticker symbol (e.g., ASTS)
        #[arg(short, long)]
        ticker: String,

        /// Company name (optional, will fetch if not provided)
        #[arg(short, long)]
        name: Option<String>,

        /// Company description (optional, will fetch if not provided)
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Classify multiple companies from a file
    ClassifyBatch {
        /// Path to CSV file with tickers
        #[arg(short, long)]
        file: String,
    },

    /// Update fundamental data for all companies in database
    UpdateFundamentals {
        /// Number of concurrent requests
        #[arg(short, long, default_value = "5")]
        concurrency: usize,
    },
}

#[derive(Subcommand)]
enum IndexCommands {
    /// Calculate index composition and weights
    Calculate {
        /// Index name (SPACEINFRA or AIINFRA)
        #[arg(short, long)]
        name: String,

        /// Save results to database
        #[arg(short, long)]
        save: bool,
    },

    /// Rebalance index for a quarter
    Rebalance {
        /// Index name (SPACEINFRA or AIINFRA)
        #[arg(short, long)]
        name: String,

        /// Quarter (e.g., Q1-2025)
        #[arg(short, long)]
        quarter: String,
    },

    /// Backtest index performance
    Backtest {
        /// Index name (SPACEINFRA or AIINFRA)
        #[arg(short, long)]
        name: String,

        /// Start date (YYYY-MM-DD)
        #[arg(short, long)]
        from: String,

        /// End date (YYYY-MM-DD, defaults to today)
        #[arg(short, long)]
        to: Option<String>,
    },

    /// List all index compositions
    List,
}

#[derive(Subcommand)]
enum DbCommands {
    /// Initialize database and run migrations
    Init,

    /// Check database status
    Status,

    /// Reset database (WARNING: deletes all data)
    Reset {
        /// Confirm reset
        #[arg(short, long)]
        confirm: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    // Set up logging
    let log_level = if cli.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .compact()
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    info!("Future Economy Indices CLI");

    // Execute commands
    match cli.command {
        Commands::Data(cmd) => match cmd {
            DataCommands::Ingest { ticker } => {
                commands::data::ingest_ticker(&ticker).await?;
            }
            DataCommands::Classify {
                ticker,
                name,
                description,
            } => {
                commands::data::classify_company(&ticker, name, description).await?;
            }
            DataCommands::ClassifyBatch { file } => {
                commands::data::classify_batch(&file).await?;
            }
            DataCommands::UpdateFundamentals { concurrency } => {
                commands::data::update_fundamentals(concurrency).await?;
            }
        },

        Commands::Index(cmd) => match cmd {
            IndexCommands::Calculate { name, save } => {
                commands::index::calculate_index(&name, save).await?;
            }
            IndexCommands::Rebalance { name, quarter } => {
                commands::index::rebalance_index(&name, &quarter).await?;
            }
            IndexCommands::Backtest { name, from, to } => {
                commands::index::backtest_index(&name, &from, to.as_deref()).await?;
            }
            IndexCommands::List => {
                commands::index::list_indices().await?;
            }
        },

        Commands::Db(cmd) => match cmd {
            DbCommands::Init => {
                commands::db::init_database().await?;
            }
            DbCommands::Status => {
                commands::db::check_status().await?;
            }
            DbCommands::Reset { confirm } => {
                if confirm {
                    commands::db::reset_database().await?;
                } else {
                    println!("⚠️  Database reset requires --confirm flag");
                    println!("   This will delete ALL data!");
                }
            }
        },
    }

    Ok(())
}
