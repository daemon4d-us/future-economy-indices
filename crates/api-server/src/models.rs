// API response models

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexInfo {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub num_constituents: i32,
    pub total_market_cap: f64,
    pub last_rebalance: Option<NaiveDate>,
    pub next_rebalance: Option<NaiveDate>,
    pub inception_date: NaiveDate,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexCompositionResponse {
    pub index_name: String,
    pub as_of_date: NaiveDate,
    pub constituents: Vec<ConstituentInfo>,
    pub total_weight: f64,
    pub num_companies: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConstituentInfo {
    pub ticker: String,
    pub company_name: String,
    pub weight: f64,
    pub market_cap: Option<i64>,
    pub space_revenue_pct: Option<f32>,
    pub segments: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceData {
    pub date: NaiveDate,
    pub index_value: f64,
    pub daily_return: Option<f64>,
    pub cumulative_return: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceResponse {
    pub index_name: String,
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
    pub data: Vec<PerformanceData>,
    pub total_return: f64,
    pub annualized_return: f64,
    pub volatility: f64,
    pub sharpe_ratio: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}
