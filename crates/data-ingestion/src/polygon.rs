// Polygon.io API client (ported from Python prototype)

use anyhow::{Context, Result};
use chrono::{Duration, NaiveDate, Utc};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration as StdDuration;
use tokio::time::sleep;
use tracing::{debug, warn};

const BASE_URL: &str = "https://api.polygon.io";
const RATE_LIMIT_DELAY_MS: u64 = 200; // Conservative 200ms delay
const MAX_RETRIES: u32 = 3;

#[derive(Clone)]
pub struct PolygonClient {
    api_key: String,
    client: Client,
    rate_limit_delay: StdDuration,
}

// Response types
#[derive(Debug, Deserialize)]
pub struct TickerDetailsResponse {
    pub results: TickerDetails,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TickerDetails {
    pub ticker: String,
    pub name: String,
    pub market_cap: Option<i64>,
    pub description: Option<String>,
    pub primary_exchange: Option<String>,
    pub locale: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchTickersResponse {
    pub results: Option<Vec<TickerSearchResult>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TickerSearchResult {
    pub ticker: String,
    pub name: String,
    pub market: Option<String>,
    pub locale: Option<String>,
    pub primary_exchange: Option<String>,
    pub active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct AggregatesResponse {
    pub results: Option<Vec<AggregateBar>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AggregateBar {
    pub t: i64,          // Timestamp (ms)
    pub o: f64,          // Open
    pub h: f64,          // High
    pub l: f64,          // Low
    pub c: f64,          // Close
    pub v: i64,          // Volume
    pub vw: Option<f64>, // Volume weighted average
    pub n: Option<i64>,  // Number of transactions
}

#[derive(Debug, Deserialize)]
pub struct FinancialsResponse {
    pub results: Option<Vec<Financial>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Financial {
    pub fiscal_year: Option<String>,
    pub fiscal_period: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub financials: Option<FinancialStatements>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FinancialStatements {
    pub income_statement: Option<IncomeStatement>,
    pub balance_sheet: Option<BalanceSheet>,
    pub cash_flow_statement: Option<CashFlowStatement>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IncomeStatement {
    pub revenues: Option<FinancialValue>,
    pub cost_of_revenue: Option<FinancialValue>,
    pub gross_profit: Option<FinancialValue>,
    pub net_income_loss: Option<FinancialValue>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BalanceSheet {
    pub assets: Option<FinancialValue>,
    pub liabilities: Option<FinancialValue>,
    pub equity: Option<FinancialValue>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CashFlowStatement {
    pub net_cash_flow_from_operating_activities: Option<FinancialValue>,
    pub net_cash_flow_from_investing_activities: Option<FinancialValue>,
    pub net_cash_flow_from_financing_activities: Option<FinancialValue>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FinancialValue {
    pub value: Option<i64>,
    pub unit: Option<String>,
}

impl PolygonClient {
    /// Create a new Polygon client with API key from environment or parameter
    pub fn new(api_key: Option<String>) -> Result<Self> {
        let api_key = api_key
            .or_else(|| std::env::var("POLYGON_API_KEY").ok())
            .context("POLYGON_API_KEY must be set in environment or passed to constructor")?;

        Ok(Self {
            api_key,
            client: Client::new(),
            rate_limit_delay: StdDuration::from_millis(RATE_LIMIT_DELAY_MS),
        })
    }

    /// Make API request with retry logic and rate limiting
    async fn make_request(
        &self,
        endpoint: &str,
        params: Option<HashMap<String, String>>,
    ) -> Result<serde_json::Value> {
        let url = format!("{}{}", BASE_URL, endpoint);
        let mut query_params = params.unwrap_or_default();
        query_params.insert("apiKey".to_string(), self.api_key.clone());

        let mut retries = 0;
        loop {
            // Rate limiting
            sleep(self.rate_limit_delay).await;

            debug!("Making request to: {}", endpoint);

            let response = self.client.get(&url).query(&query_params).send().await;

            match response {
                Ok(resp) => {
                    let status = resp.status();

                    if status.is_success() {
                        let json = resp
                            .json::<serde_json::Value>()
                            .await
                            .context("Failed to parse JSON response")?;
                        return Ok(json);
                    }

                    // Handle rate limiting
                    if status == StatusCode::TOO_MANY_REQUESTS && retries < MAX_RETRIES {
                        let backoff = 2u64.pow(retries) * 1000; // Exponential backoff
                        warn!("Rate limited, backing off for {}ms", backoff);
                        sleep(StdDuration::from_millis(backoff)).await;
                        retries += 1;
                        continue;
                    }

                    // Handle server errors with retry
                    if status.is_server_error() && retries < MAX_RETRIES {
                        warn!(
                            "Server error {}, retrying... ({}/{})",
                            status,
                            retries + 1,
                            MAX_RETRIES
                        );
                        sleep(StdDuration::from_secs(2)).await;
                        retries += 1;
                        continue;
                    }

                    anyhow::bail!("HTTP error: {}", status);
                }
                Err(e) => {
                    if retries < MAX_RETRIES {
                        warn!(
                            "Request failed: {}, retrying... ({}/{})",
                            e,
                            retries + 1,
                            MAX_RETRIES
                        );
                        sleep(StdDuration::from_secs(2)).await;
                        retries += 1;
                        continue;
                    }
                    return Err(e).context("HTTP request failed");
                }
            }
        }
    }

    /// Get detailed information about a ticker
    pub async fn get_ticker_details(&self, ticker: &str) -> Result<TickerDetails> {
        let endpoint = format!("/v3/reference/tickers/{}", ticker);
        let json = self.make_request(&endpoint, None).await?;

        let response: TickerDetailsResponse =
            serde_json::from_value(json).context("Failed to parse ticker details response")?;

        Ok(response.results)
    }

    /// Search for tickers matching criteria
    pub async fn search_tickers(
        &self,
        market: Option<&str>,
        exchange: Option<&str>,
        active: bool,
        limit: u32,
    ) -> Result<Vec<TickerSearchResult>> {
        let endpoint = "/v3/reference/tickers";
        let mut params = HashMap::new();

        if let Some(m) = market {
            params.insert("market".to_string(), m.to_string());
        }
        if let Some(ex) = exchange {
            params.insert("exchange".to_string(), ex.to_string());
        }
        params.insert("active".to_string(), active.to_string());
        params.insert("limit".to_string(), limit.to_string());

        let json = self.make_request(endpoint, Some(params)).await?;

        let response: SearchTickersResponse =
            serde_json::from_value(json).context("Failed to parse search response")?;

        Ok(response.results.unwrap_or_default())
    }

    /// Get aggregate bars (OHLCV) for a ticker
    pub async fn get_aggregates(
        &self,
        ticker: &str,
        multiplier: u32,
        timespan: &str,
        from_date: Option<NaiveDate>,
        to_date: Option<NaiveDate>,
        limit: u32,
    ) -> Result<Vec<AggregateBar>> {
        let from =
            from_date.unwrap_or_else(|| (Utc::now() - Duration::days(365)).naive_utc().date());
        let to = to_date.unwrap_or_else(|| Utc::now().naive_utc().date());

        let endpoint = format!(
            "/v2/aggs/ticker/{}/range/{}/{}/{}/{}",
            ticker,
            multiplier,
            timespan,
            from.format("%Y-%m-%d"),
            to.format("%Y-%m-%d")
        );

        let mut params = HashMap::new();
        params.insert("limit".to_string(), limit.to_string());
        params.insert("adjusted".to_string(), "true".to_string());

        let json = self.make_request(&endpoint, Some(params)).await?;

        let response: AggregatesResponse =
            serde_json::from_value(json).context("Failed to parse aggregates response")?;

        Ok(response.results.unwrap_or_default())
    }

    /// Get financial data (income statement, balance sheet, cash flow)
    pub async fn get_financials(
        &self,
        ticker: &str,
        timeframe: &str,
        limit: u32,
    ) -> Result<Vec<Financial>> {
        let endpoint = "/vX/reference/financials";
        let mut params = HashMap::new();
        params.insert("ticker".to_string(), ticker.to_string());
        params.insert("timeframe".to_string(), timeframe.to_string());
        params.insert("limit".to_string(), limit.to_string());

        let json = self.make_request(endpoint, Some(params)).await?;

        let response: FinancialsResponse =
            serde_json::from_value(json).context("Failed to parse financials response")?;

        Ok(response.results.unwrap_or_default())
    }

    /// Get current market capitalization for a ticker
    pub async fn get_market_cap(&self, ticker: &str) -> Result<Option<i64>> {
        match self.get_ticker_details(ticker).await {
            Ok(details) => Ok(details.market_cap),
            Err(e) => {
                warn!("Error fetching market cap for {}: {}", ticker, e);
                Ok(None)
            }
        }
    }

    /// Calculate revenue growth rate from financials
    pub fn calculate_revenue_growth(financials: &[Financial]) -> Option<f32> {
        if financials.len() < 2 {
            return None;
        }

        let latest = financials.first()?;
        let previous = financials.get(1)?;

        let latest_revenue = latest
            .financials
            .as_ref()?
            .income_statement
            .as_ref()?
            .revenues
            .as_ref()?
            .value? as f32;

        let previous_revenue = previous
            .financials
            .as_ref()?
            .income_statement
            .as_ref()?
            .revenues
            .as_ref()?
            .value? as f32;

        if previous_revenue == 0.0 {
            return None;
        }

        Some(((latest_revenue - previous_revenue) / previous_revenue) * 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_polygon_client_creation() {
        // This test requires POLYGON_API_KEY in environment
        if std::env::var("POLYGON_API_KEY").is_ok() {
            let client = PolygonClient::new(None);
            assert!(client.is_ok());
        }
    }

    #[test]
    fn test_revenue_growth_calculation() {
        let financials = vec![
            Financial {
                fiscal_year: Some("2024".to_string()),
                fiscal_period: None,
                start_date: None,
                end_date: None,
                financials: Some(FinancialStatements {
                    income_statement: Some(IncomeStatement {
                        revenues: Some(FinancialValue {
                            value: Some(100_000_000),
                            unit: Some("USD".to_string()),
                        }),
                        cost_of_revenue: None,
                        gross_profit: None,
                        net_income_loss: None,
                    }),
                    balance_sheet: None,
                    cash_flow_statement: None,
                }),
            },
            Financial {
                fiscal_year: Some("2023".to_string()),
                fiscal_period: None,
                start_date: None,
                end_date: None,
                financials: Some(FinancialStatements {
                    income_statement: Some(IncomeStatement {
                        revenues: Some(FinancialValue {
                            value: Some(80_000_000),
                            unit: Some("USD".to_string()),
                        }),
                        cost_of_revenue: None,
                        gross_profit: None,
                        net_income_loss: None,
                    }),
                    balance_sheet: None,
                    cash_flow_statement: None,
                }),
            },
        ];

        let growth = PolygonClient::calculate_revenue_growth(&financials);
        assert!(growth.is_some());
        assert!((growth.unwrap() - 25.0).abs() < 0.01); // 25% growth
    }
}
