// Database schema helpers and queries

use anyhow::Result;
use chrono::NaiveDate;
use sqlx::PgPool;

use crate::models::{Company, Fundamental, IndexComposition, IndexPerformance};

// ============================================================================
// Company Queries
// ============================================================================

/// Insert or update a company
pub async fn upsert_company(pool: &PgPool, company: &Company) -> Result<i32> {
    let row = sqlx::query!(
        r#"
        INSERT INTO companies (ticker, name, description, market_cap, space_score, ai_score, segments, last_classified_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (ticker)
        DO UPDATE SET
            name = EXCLUDED.name,
            description = EXCLUDED.description,
            market_cap = EXCLUDED.market_cap,
            space_score = EXCLUDED.space_score,
            ai_score = EXCLUDED.ai_score,
            segments = EXCLUDED.segments,
            last_classified_at = EXCLUDED.last_classified_at,
            updated_at = NOW()
        RETURNING id
        "#,
        company.ticker,
        company.name,
        company.description,
        company.market_cap,
        company.space_score,
        company.ai_score,
        company.segments.as_deref(),
        company.last_classified_at
    )
    .fetch_one(pool)
    .await?;

    Ok(row.id)
}

/// Get company by ticker
pub async fn get_company_by_ticker(pool: &PgPool, ticker: &str) -> Result<Option<Company>> {
    let company = sqlx::query_as!(
        Company,
        r#"
        SELECT id, ticker, name, description, market_cap, space_score, ai_score,
               segments as "segments: Vec<String>", last_classified_at, created_at, updated_at
        FROM companies
        WHERE ticker = $1
        "#,
        ticker
    )
    .fetch_optional(pool)
    .await?;

    Ok(company)
}

/// Get all companies
pub async fn get_all_companies(pool: &PgPool) -> Result<Vec<Company>> {
    let companies = sqlx::query_as!(
        Company,
        r#"
        SELECT id, ticker, name, description, market_cap, space_score, ai_score,
               segments as "segments: Vec<String>", last_classified_at, created_at, updated_at
        FROM companies
        ORDER BY ticker
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(companies)
}

/// Get companies by minimum space score
pub async fn get_companies_by_space_score(pool: &PgPool, min_score: f32) -> Result<Vec<Company>> {
    let companies = sqlx::query_as!(
        Company,
        r#"
        SELECT id, ticker, name, description, market_cap, space_score, ai_score,
               segments as "segments: Vec<String>", last_classified_at, created_at, updated_at
        FROM companies
        WHERE space_score >= $1
        ORDER BY space_score DESC
        "#,
        min_score
    )
    .fetch_all(pool)
    .await?;

    Ok(companies)
}

// ============================================================================
// Fundamental Queries
// ============================================================================

/// Insert fundamental data
pub async fn insert_fundamental(pool: &PgPool, fundamental: &Fundamental) -> Result<i32> {
    let row = sqlx::query!(
        r#"
        INSERT INTO fundamentals (
            company_id, date, revenue, revenue_growth_yoy, revenue_growth_3y_cagr,
            market_cap, price, volume
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id
        "#,
        fundamental.company_id,
        fundamental.date,
        fundamental.revenue,
        fundamental.revenue_growth_yoy,
        fundamental.revenue_growth_3y_cagr,
        fundamental.market_cap,
        fundamental.price,
        fundamental.volume
    )
    .fetch_one(pool)
    .await?;

    Ok(row.id)
}

/// Get latest fundamental for a company
pub async fn get_latest_fundamental(pool: &PgPool, company_id: i32) -> Result<Option<Fundamental>> {
    let fundamental = sqlx::query_as!(
        Fundamental,
        r#"
        SELECT id as "id!", company_id as "company_id!", date, revenue, revenue_growth_yoy, revenue_growth_3y_cagr,
               market_cap, price, volume, created_at
        FROM fundamentals
        WHERE company_id = $1
        ORDER BY date DESC
        LIMIT 1
        "#,
        company_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(fundamental)
}

/// Get fundamentals for a company
pub async fn get_fundamentals_by_company(
    pool: &PgPool,
    company_id: i32,
    limit: i64,
) -> Result<Vec<Fundamental>> {
    let fundamentals = sqlx::query_as!(
        Fundamental,
        r#"
        SELECT id as "id!", company_id as "company_id!", date, revenue, revenue_growth_yoy, revenue_growth_3y_cagr,
               market_cap, price, volume, created_at
        FROM fundamentals
        WHERE company_id = $1
        ORDER BY date DESC
        LIMIT $2
        "#,
        company_id,
        limit
    )
    .fetch_all(pool)
    .await?;

    Ok(fundamentals)
}

// ============================================================================
// Index Composition Queries
// ============================================================================

/// Insert index composition
pub async fn insert_index_composition(
    pool: &PgPool,
    composition: &IndexComposition,
) -> Result<i32> {
    let row = sqlx::query!(
        r#"
        INSERT INTO index_compositions (
            index_name, company_id, weight, rebalance_date, rank,
            space_revenue_pct, revenue_growth_rate, reason_included
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id
        "#,
        composition.index_name,
        composition.company_id,
        composition.weight,
        composition.rebalance_date,
        composition.rank,
        composition.space_revenue_pct,
        composition.revenue_growth_rate,
        composition.reason_included
    )
    .fetch_one(pool)
    .await?;

    Ok(row.id)
}

/// Get current index composition (latest rebalance_date)
pub async fn get_current_index_composition(
    pool: &PgPool,
    index_name: &str,
) -> Result<Vec<IndexComposition>> {
    let compositions = sqlx::query_as!(
        IndexComposition,
        r#"
        SELECT ic.id as "id!", ic.index_name, ic.company_id as "company_id!", ic.weight, ic.rebalance_date,
               ic.rank, ic.space_revenue_pct, ic.revenue_growth_rate, ic.reason_included, ic.created_at
        FROM index_compositions ic
        WHERE ic.index_name = $1
        AND ic.rebalance_date = (
            SELECT MAX(rebalance_date)
            FROM index_compositions
            WHERE index_name = $1
        )
        ORDER BY ic.weight DESC
        "#,
        index_name
    )
    .fetch_all(pool)
    .await?;

    Ok(compositions)
}

/// Get index composition as of a specific date
pub async fn get_index_composition_as_of(
    pool: &PgPool,
    index_name: &str,
    rebalance_date: NaiveDate,
) -> Result<Vec<IndexComposition>> {
    let compositions = sqlx::query_as!(
        IndexComposition,
        r#"
        SELECT id as "id!", index_name, company_id as "company_id!", weight, rebalance_date,
               rank, space_revenue_pct, revenue_growth_rate, reason_included, created_at
        FROM index_compositions
        WHERE index_name = $1 AND rebalance_date = $2
        ORDER BY weight DESC
        "#,
        index_name,
        rebalance_date
    )
    .fetch_all(pool)
    .await?;

    Ok(compositions)
}

/// Get all rebalance dates for an index
pub async fn get_index_rebalance_dates(pool: &PgPool, index_name: &str) -> Result<Vec<NaiveDate>> {
    let dates = sqlx::query!(
        r#"
        SELECT DISTINCT rebalance_date
        FROM index_compositions
        WHERE index_name = $1
        ORDER BY rebalance_date DESC
        "#,
        index_name
    )
    .fetch_all(pool)
    .await?;

    Ok(dates.into_iter().map(|r| r.rebalance_date).collect())
}

// ============================================================================
// Index Performance Queries
// ============================================================================

/// Insert index performance
pub async fn insert_index_performance(
    pool: &PgPool,
    performance: &IndexPerformance,
) -> Result<i32> {
    let row = sqlx::query!(
        r#"
        INSERT INTO index_performance (
            index_name, date, value, daily_return
        )
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (index_name, date)
        DO UPDATE SET
            value = EXCLUDED.value,
            daily_return = EXCLUDED.daily_return
        RETURNING id
        "#,
        performance.index_name,
        performance.date,
        performance.value,
        performance.daily_return
    )
    .fetch_one(pool)
    .await?;

    Ok(row.id)
}

/// Get index performance for date range
pub async fn get_index_performance(
    pool: &PgPool,
    index_name: &str,
    from_date: NaiveDate,
    to_date: NaiveDate,
) -> Result<Vec<IndexPerformance>> {
    let performance = sqlx::query_as!(
        IndexPerformance,
        r#"
        SELECT id, index_name, date, value, daily_return, created_at
        FROM index_performance
        WHERE index_name = $1
        AND date >= $2
        AND date <= $3
        ORDER BY date ASC
        "#,
        index_name,
        from_date,
        to_date
    )
    .fetch_all(pool)
    .await?;

    Ok(performance)
}

/// Get latest index performance
pub async fn get_latest_index_performance(
    pool: &PgPool,
    index_name: &str,
) -> Result<Option<IndexPerformance>> {
    let performance = sqlx::query_as!(
        IndexPerformance,
        r#"
        SELECT id, index_name, date, value, daily_return, created_at
        FROM index_performance
        WHERE index_name = $1
        ORDER BY date DESC
        LIMIT 1
        "#,
        index_name
    )
    .fetch_optional(pool)
    .await?;

    Ok(performance)
}

// ============================================================================
// Composite Queries (joining multiple tables)
// ============================================================================

/// Get index composition with company details
#[derive(Debug)]
pub struct CompositionWithCompany {
    pub ticker: String,
    pub company_name: String,
    pub weight: f64,
    pub market_cap: Option<i64>,
    pub space_score: Option<f64>,
    pub segments: Option<Vec<String>>,
}

pub async fn get_index_composition_with_companies(
    pool: &PgPool,
    index_name: &str,
) -> Result<Vec<CompositionWithCompany>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            c.ticker,
            c.name as company_name,
            ic.weight,
            c.market_cap,
            c.space_score,
            c.segments
        FROM index_compositions ic
        JOIN companies c ON ic.company_id = c.id
        WHERE ic.index_name = $1
        AND ic.rebalance_date = (
            SELECT MAX(rebalance_date)
            FROM index_compositions
            WHERE index_name = $1
        )
        ORDER BY ic.weight DESC
        "#,
        index_name
    )
    .fetch_all(pool)
    .await?;

    let compositions = rows
        .into_iter()
        .map(|row| CompositionWithCompany {
            ticker: row.ticker,
            company_name: row.company_name,
            weight: row.weight as f64,
            market_cap: row.market_cap,
            space_score: row.space_score.map(|s| s as f64),
            segments: row.segments,
        })
        .collect();

    Ok(compositions)
}

/// Get index metadata
#[derive(Debug)]
pub struct IndexMetadata {
    pub index_name: String,
    pub num_constituents: i32,
    pub total_market_cap: f64,
    pub last_rebalance: Option<NaiveDate>,
    pub latest_value: Option<f64>,
    pub latest_return: Option<f64>,
}

pub async fn get_index_metadata(pool: &PgPool, index_name: &str) -> Result<Option<IndexMetadata>> {
    // Get latest composition date
    let latest_composition_date = sqlx::query_scalar!(
        r#"
        SELECT MAX(rebalance_date) as "max_date"
        FROM index_compositions
        WHERE index_name = $1
        "#,
        index_name
    )
    .fetch_one(pool)
    .await?;

    if latest_composition_date.is_none() {
        return Ok(None);
    }

    let last_rebalance = latest_composition_date.unwrap();

    // Get composition with company data
    let composition_data = sqlx::query!(
        r#"
        SELECT
            COUNT(*) as "num_companies!",
            COALESCE(SUM(c.market_cap * ic.weight::float8), 0.0)::float8 as "total_market_cap!"
        FROM index_compositions ic
        JOIN companies c ON ic.company_id = c.id
        WHERE ic.index_name = $1 AND ic.rebalance_date = $2
        "#,
        index_name,
        last_rebalance
    )
    .fetch_one(pool)
    .await?;

    // Get latest performance
    let latest_performance = get_latest_index_performance(pool, index_name).await?;

    Ok(Some(IndexMetadata {
        index_name: index_name.to_string(),
        num_constituents: composition_data.num_companies as i32,
        total_market_cap: composition_data.total_market_cap,
        last_rebalance: Some(last_rebalance),
        latest_value: latest_performance.as_ref().map(|p| p.value as f64),
        latest_return: None, // We don't have total_return field in the table
    }))
}
