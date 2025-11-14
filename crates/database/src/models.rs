// Database models

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Company {
    pub id: i32,
    pub ticker: String,
    pub name: String,
    pub description: Option<String>,
    pub market_cap: Option<i64>,
    pub space_score: Option<f32>,
    pub ai_score: Option<f32>,
    pub segments: Option<Vec<String>>,
    pub last_classified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Fundamental {
    pub id: i32,
    pub company_id: i32,
    pub date: NaiveDate,
    pub revenue: Option<i64>,
    pub revenue_growth_yoy: Option<f32>,
    pub revenue_growth_3y_cagr: Option<f32>,
    pub market_cap: Option<i64>,
    pub price: Option<f32>,
    pub volume: Option<i64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IndexComposition {
    pub id: i32,
    pub index_name: String,
    pub rebalance_date: NaiveDate,
    pub company_id: i32,
    pub weight: f32,
    pub rank: Option<i32>,
    pub space_revenue_pct: Option<f32>,
    pub revenue_growth_rate: Option<f32>,
    pub reason_included: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IndexPerformance {
    pub id: i32,
    pub index_name: String,
    pub date: NaiveDate,
    pub value: f32,
    pub daily_return: Option<f32>,
    pub created_at: DateTime<Utc>,
}
