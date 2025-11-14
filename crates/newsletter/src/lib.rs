// Newsletter generation and email delivery

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

pub mod templates;
pub mod convertkit;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsletterData {
    pub index_name: String,
    pub period: String,
    pub quarter: String,
    pub total_return: f64,
    pub ytd_return: f64,
    pub vs_sp500: f64,
    pub top_holdings: Vec<HoldingData>,
    pub rebalancing_changes: RebalancingChanges,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoldingData {
    pub ticker: String,
    pub company_name: String,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebalancingChanges {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub date: NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscriber {
    pub email: String,
    pub first_name: Option<String>,
    pub tier: SubscriptionTier,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SubscriptionTier {
    Free,
    Paid,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_newsletter_data_creation() {
        let data = NewsletterData {
            index_name: "SPACEINFRA".to_string(),
            period: "Q4 2024".to_string(),
            quarter: "Q4".to_string(),
            total_return: 25.5,
            ytd_return: 83.0,
            vs_sp500: 55.0,
            top_holdings: vec![],
            rebalancing_changes: RebalancingChanges {
                added: vec![],
                removed: vec![],
                date: NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            },
        };
        assert_eq!(data.index_name, "SPACEINFRA");
    }
}
