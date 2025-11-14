// Email templates for newsletters

use crate::NewsletterData;

pub fn generate_quarterly_report(data: &NewsletterData) -> String {
    format!(
        r#"
# {} - {} Performance Update

## QUARTERLY PERFORMANCE
- Return: {:.1}%
- YTD: {:.1}%
- vs S&P 500: {:.1}%

## TOP HOLDINGS
{}

## REBALANCING CHANGES
Added: {}
Removed: {}

---
Read full report at https://futureeconomy.indices/{}

Upgrade to paid for weekly updates!
"#,
        data.index_name,
        data.period,
        data.total_return,
        data.ytd_return,
        data.vs_sp500,
        format_holdings(&data.top_holdings),
        if data.rebalancing_changes.added.is_empty() {
            "None".to_string()
        } else {
            data.rebalancing_changes.added.join(", ")
        },
        if data.rebalancing_changes.removed.is_empty() {
            "None".to_string()
        } else {
            data.rebalancing_changes.removed.join(", ")
        },
        data.index_name.to_lowercase()
    )
}

fn format_holdings(holdings: &[crate::HoldingData]) -> String {
    holdings
        .iter()
        .enumerate()
        .map(|(i, h)| {
            format!(
                "{}. {} - {} ({:.1}%)",
                i + 1,
                h.ticker,
                h.company_name,
                h.weight * 100.0
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HoldingData, RebalancingChanges};
    use chrono::NaiveDate;

    #[test]
    fn test_template_generation() {
        let data = NewsletterData {
            index_name: "SPACEINFRA".to_string(),
            period: "Q4 2024".to_string(),
            quarter: "Q4".to_string(),
            total_return: 25.5,
            ytd_return: 83.0,
            vs_sp500: 55.0,
            top_holdings: vec![HoldingData {
                ticker: "RKLB".to_string(),
                company_name: "Rocket Lab".to_string(),
                weight: 0.10,
            }],
            rebalancing_changes: RebalancingChanges {
                added: vec!["LUNR".to_string()],
                removed: vec![],
                date: NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            },
        };

        let email = generate_quarterly_report(&data);
        assert!(email.contains("SPACEINFRA"));
        assert!(email.contains("25.5%"));
        assert!(email.contains("RKLB"));
    }
}
