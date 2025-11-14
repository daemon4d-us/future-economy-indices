// Index API endpoints

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::NaiveDate;
use serde::Deserialize;
use tracing::error;

use crate::{
    models::{
        ConstituentInfo, ErrorResponse, IndexCompositionResponse, IndexInfo, PerformanceData,
        PerformanceResponse,
    },
    state::AppState,
};

// Import database query functions
use database;

#[derive(Debug, Deserialize)]
pub struct PerformanceQuery {
    pub from: Option<String>,
    pub to: Option<String>,
}

/// GET /api/indices
/// List all available indices
pub async fn list_indices(
    State(state): State<AppState>,
) -> Result<Json<Vec<IndexInfo>>, (StatusCode, Json<ErrorResponse>)> {
    let mut indices = Vec::new();

    // Query metadata for known indices
    for index_name in &["SPACEINFRA", "AIINFRA"] {
        match database::get_index_metadata(&state.db, index_name).await {
            Ok(Some(metadata)) => {
                let (display_name, description, inception_date, next_rebalance) = match *index_name {
                    "SPACEINFRA" => (
                        "Space Infrastructure Index",
                        "Tracks companies in the space infrastructure industry including launch, satellites, ground systems, and components.",
                        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                        Some(NaiveDate::from_ymd_opt(2025, 3, 31).unwrap()),
                    ),
                    "AIINFRA" => (
                        "AI Infrastructure Index",
                        "Tracks companies building the infrastructure for artificial intelligence.",
                        NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                        Some(NaiveDate::from_ymd_opt(2025, 3, 31).unwrap()),
                    ),
                    _ => continue,
                };

                indices.push(IndexInfo {
                    name: metadata.index_name,
                    display_name: display_name.to_string(),
                    description: description.to_string(),
                    num_constituents: metadata.num_constituents,
                    total_market_cap: metadata.total_market_cap,
                    last_rebalance: metadata.last_rebalance,
                    next_rebalance,
                    inception_date,
                });
            }
            Ok(None) => {
                // Index exists but has no data yet, return placeholder
                if *index_name == "AIINFRA" {
                    indices.push(IndexInfo {
                        name: "AIINFRA".to_string(),
                        display_name: "AI Infrastructure Index".to_string(),
                        description: "Tracks companies building the infrastructure for artificial intelligence.".to_string(),
                        num_constituents: 0,
                        total_market_cap: 0.0,
                        last_rebalance: None,
                        next_rebalance: Some(NaiveDate::from_ymd_opt(2025, 3, 31).unwrap()),
                        inception_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                    });
                }
            }
            Err(e) => {
                error!("Failed to fetch metadata for {}: {:?}", index_name, e);
            }
        }
    }

    Ok(Json(indices))
}

/// GET /api/indices/:name
/// Get index details
pub async fn get_index(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<IndexInfo>, (StatusCode, Json<ErrorResponse>)> {
    let index_name_upper = name.to_uppercase();

    // Validate index name
    if !["SPACEINFRA", "AIINFRA"].contains(&index_name_upper.as_str()) {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "NOT_FOUND".to_string(),
                message: format!("Index '{}' not found", name),
            }),
        ));
    }

    match database::get_index_metadata(&state.db, &index_name_upper).await {
        Ok(Some(metadata)) => {
            let (display_name, description, inception_date, next_rebalance) = match index_name_upper.as_str() {
                "SPACEINFRA" => (
                    "Space Infrastructure Index",
                    "Tracks companies in the space infrastructure industry including launch, satellites, ground systems, and components.",
                    NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                    Some(NaiveDate::from_ymd_opt(2025, 3, 31).unwrap()),
                ),
                "AIINFRA" => (
                    "AI Infrastructure Index",
                    "Tracks companies building the infrastructure for artificial intelligence.",
                    NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                    Some(NaiveDate::from_ymd_opt(2025, 3, 31).unwrap()),
                ),
                _ => return Err((
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse {
                        error: "NOT_FOUND".to_string(),
                        message: format!("Index '{}' not found", name),
                    }),
                )),
            };

            Ok(Json(IndexInfo {
                name: metadata.index_name,
                display_name: display_name.to_string(),
                description: description.to_string(),
                num_constituents: metadata.num_constituents,
                total_market_cap: metadata.total_market_cap,
                last_rebalance: metadata.last_rebalance,
                next_rebalance,
                inception_date,
            }))
        }
        Ok(None) => {
            // Index has no composition data yet
            if index_name_upper == "AIINFRA" {
                Ok(Json(IndexInfo {
                    name: "AIINFRA".to_string(),
                    display_name: "AI Infrastructure Index".to_string(),
                    description:
                        "Tracks companies building the infrastructure for artificial intelligence."
                            .to_string(),
                    num_constituents: 0,
                    total_market_cap: 0.0,
                    last_rebalance: None,
                    next_rebalance: Some(NaiveDate::from_ymd_opt(2025, 3, 31).unwrap()),
                    inception_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                }))
            } else {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse {
                        error: "NOT_FOUND".to_string(),
                        message: format!("Index '{}' has no data", name),
                    }),
                ))
            }
        }
        Err(e) => {
            error!("Database error fetching index {}: {:?}", name, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "INTERNAL_SERVER_ERROR".to_string(),
                    message: "Failed to fetch index data".to_string(),
                }),
            ))
        }
    }
}

/// GET /api/indices/:name/composition
/// Get current index composition
pub async fn get_composition(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<IndexCompositionResponse>, (StatusCode, Json<ErrorResponse>)> {
    let index_name_upper = name.to_uppercase();

    // Validate index name
    if !["SPACEINFRA", "AIINFRA"].contains(&index_name_upper.as_str()) {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "NOT_FOUND".to_string(),
                message: format!("Index '{}' not found", name),
            }),
        ));
    }

    // Fetch composition with company details
    match database::get_index_composition_with_companies(&state.db, &index_name_upper).await {
        Ok(compositions) => {
            if compositions.is_empty() {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse {
                        error: "NOT_FOUND".to_string(),
                        message: format!("Index '{}' has no composition data", name),
                    }),
                ));
            }

            // Get rebalance date from the index
            let as_of_date =
                match database::get_index_rebalance_dates(&state.db, &index_name_upper).await {
                    Ok(dates) if !dates.is_empty() => dates[0],
                    _ => NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
                };

            let total_weight: f64 = compositions.iter().map(|c| c.weight).sum();
            let num_companies = compositions.len() as i32;

            let constituents: Vec<ConstituentInfo> = compositions
                .into_iter()
                .map(|c| ConstituentInfo {
                    ticker: c.ticker,
                    company_name: c.company_name,
                    weight: c.weight,
                    market_cap: c.market_cap,
                    space_revenue_pct: c.space_score.map(|s| (s * 100.0) as f32),
                    segments: c.segments,
                })
                .collect();

            Ok(Json(IndexCompositionResponse {
                index_name: index_name_upper,
                as_of_date,
                constituents,
                total_weight,
                num_companies,
            }))
        }
        Err(e) => {
            error!("Database error fetching composition for {}: {:?}", name, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "INTERNAL_SERVER_ERROR".to_string(),
                    message: "Failed to fetch composition data".to_string(),
                }),
            ))
        }
    }
}

/// GET /api/indices/:name/performance
/// Get historical performance data
pub async fn get_performance(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Query(query): Query<PerformanceQuery>,
) -> Result<Json<PerformanceResponse>, (StatusCode, Json<ErrorResponse>)> {
    let index_name_upper = name.to_uppercase();

    // Validate index name
    if !["SPACEINFRA", "AIINFRA"].contains(&index_name_upper.as_str()) {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "NOT_FOUND".to_string(),
                message: format!("Index '{}' not found", name),
            }),
        ));
    }

    // Parse dates or use defaults
    let from_date = query
        .from
        .and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok())
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

    let to_date = query
        .to
        .and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok())
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(2024, 12, 31).unwrap());

    // Fetch performance data from database
    match database::get_index_performance(&state.db, &index_name_upper, from_date, to_date).await {
        Ok(performance_records) => {
            if performance_records.is_empty() {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse {
                        error: "NOT_FOUND".to_string(),
                        message: format!("No performance data found for index '{}'", name),
                    }),
                ));
            }

            // Calculate cumulative returns
            let first_value = performance_records.first().unwrap().value;
            let last_value = performance_records.last().unwrap().value;

            let data: Vec<PerformanceData> = performance_records
                .iter()
                .map(|record| {
                    let cumulative_return = ((record.value / first_value) - 1.0) * 100.0;
                    PerformanceData {
                        date: record.date,
                        index_value: record.value as f64,
                        daily_return: record.daily_return.map(|r| r as f64),
                        cumulative_return: Some(cumulative_return as f64),
                    }
                })
                .collect();

            // Calculate statistics
            let total_return = ((last_value / first_value) - 1.0) * 100.0;
            let num_days = (to_date - from_date).num_days() as f32;
            let years = num_days / 365.0;
            let annualized_return = if years > 0.0 {
                ((last_value / first_value).powf(1.0 / years) - 1.0) * 100.0
            } else {
                total_return
            };

            // Calculate volatility (daily return stddev * sqrt(252))
            let returns: Vec<f32> = performance_records
                .iter()
                .filter_map(|r| r.daily_return)
                .collect();

            let volatility = if returns.len() > 1 {
                let mean = returns.iter().sum::<f32>() / returns.len() as f32;
                let variance = returns.iter().map(|r| (r - mean).powi(2)).sum::<f32>()
                    / (returns.len() - 1) as f32;
                (variance.sqrt() * 252.0_f32.sqrt()) * 100.0
            } else {
                0.0
            };

            // Calculate Sharpe ratio (assuming 0 risk-free rate)
            let sharpe_ratio = if volatility > 0.0 {
                Some((annualized_return / volatility) as f64)
            } else {
                None
            };

            Ok(Json(PerformanceResponse {
                index_name: index_name_upper,
                from_date,
                to_date,
                data,
                total_return: total_return as f64,
                annualized_return: annualized_return as f64,
                volatility: volatility as f64,
                sharpe_ratio,
            }))
        }
        Err(e) => {
            error!("Database error fetching performance for {}: {:?}", name, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "INTERNAL_SERVER_ERROR".to_string(),
                    message: "Failed to fetch performance data".to_string(),
                }),
            ))
        }
    }
}
