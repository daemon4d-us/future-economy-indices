// Data management commands

use ai_classifier::{AnthropicClassifier, CompanyInfo};
use anyhow::{Context, Result};
use data_ingestion::PolygonClient;
use tracing::{info, warn};

/// Fetch and ingest ticker data from Polygon.io
pub async fn ingest_ticker(ticker: &str) -> Result<()> {
    info!("Ingesting data for ticker: {}", ticker);

    let client = PolygonClient::new(None)
        .context("Failed to create Polygon client - check POLYGON_API_KEY")?;

    // Fetch ticker details
    info!("Fetching ticker details...");
    let details = client.get_ticker_details(ticker).await?;

    println!("\n[+] Ticker Details:");
    println!("   Ticker: {}", details.ticker);
    println!("   Name: {}", details.name);
    println!(
        "   Market Cap: ${:.2}B",
        details.market_cap.unwrap_or(0) as f64 / 1e9
    );
    if let Some(desc) = &details.description {
        println!("   Description: {}", &desc[..desc.len().min(100)]);
    }

    // Fetch financials
    info!("Fetching financials...");
    let financials = client.get_financials(ticker, "annual", 2).await?;

    println!("\n[+] Financials:");
    println!("   {} annual reports found", financials.len());

    if financials.len() >= 2 {
        if let Some(growth) = PolygonClient::calculate_revenue_growth(&financials) {
            println!("   Revenue Growth: {:.1}%", growth);
        }
    }

    // Fetch recent price data
    info!("Fetching price data...");
    let aggregates = client
        .get_aggregates(ticker, 1, "day", None, None, 30)
        .await?;

    if !aggregates.is_empty() {
        let latest = &aggregates[aggregates.len() - 1];
        println!("\n[+] Latest Price:");
        println!("   Close: ${:.2}", latest.c);
        println!("   Volume: {}", latest.v);
    }

    println!("\n[+] Data ingestion complete for {}", ticker);

    Ok(())
}

/// Classify a company using AI
pub async fn classify_company(
    ticker: &str,
    name: Option<String>,
    description: Option<String>,
) -> Result<()> {
    info!("Classifying company: {}", ticker);

    // If name/description not provided, fetch from Polygon
    let (company_name, company_desc) = if name.is_none() || description.is_none() {
        info!("Fetching company details from Polygon.io...");
        let client = PolygonClient::new(None)?;
        let details = client.get_ticker_details(ticker).await?;
        (
            name.unwrap_or(details.name),
            description.unwrap_or(details.description.unwrap_or_default()),
        )
    } else {
        (name.unwrap(), description.unwrap())
    };

    // Classify using AI
    info!("Classifying with AI...");
    let classifier = AnthropicClassifier::new(None)
        .context("Failed to create AI classifier - check ANTHROPIC_API_KEY")?;

    let result = classifier
        .classify_company(ticker, &company_name, &company_desc, None)
        .await?;

    println!("\n[AI] Classification Results:");
    println!("   Ticker: {}", result.ticker);
    println!("   Company: {}", result.company_name);
    println!(
        "   Space Related: {}",
        if result.is_space_related { "YES" } else { "NO" }
    );
    println!("   Space Revenue %: {:.1}%", result.space_revenue_pct);
    println!("   Confidence: {}", result.confidence);
    println!("   Segments: {}", result.segments.join(", "));
    println!("\n   Reasoning: {}", result.reasoning);

    Ok(())
}

/// Classify multiple companies from a CSV file
pub async fn classify_batch(file_path: &str) -> Result<()> {
    info!("Batch classification from file: {}", file_path);

    // Read CSV file
    let contents = std::fs::read_to_string(file_path).context("Failed to read CSV file")?;

    let mut companies = Vec::new();

    for (i, line) in contents.lines().enumerate() {
        // Skip header
        if i == 0 && (line.contains("ticker") || line.contains("symbol")) {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 2 {
            warn!("Skipping invalid line {}: {}", i + 1, line);
            continue;
        }

        let ticker = parts[0].trim().to_string();
        let name = parts[1].trim().to_string();
        let description = if parts.len() > 2 {
            parts[2].trim().to_string()
        } else {
            String::new()
        };

        companies.push(CompanyInfo {
            ticker,
            name,
            description,
            context: None,
        });
    }

    println!("\n[+] Found {} companies to classify", companies.len());

    // Classify batch
    let classifier = AnthropicClassifier::new(None)?;
    let results = classifier.batch_classify(companies, true).await;

    // Print summary
    let space_companies: Vec<_> = results.iter().filter(|r| r.is_space_related).collect();

    println!("\n[+] Batch classification complete!");
    println!("   Total: {}", results.len());
    println!("   Space-related: {}", space_companies.len());
    println!("   Non-space: {}", results.len() - space_companies.len());

    // Print space companies
    if !space_companies.is_empty() {
        println!("\n[+] Space Companies:");
        for r in space_companies {
            println!(
                "   {} - {} ({}% space revenue)",
                r.ticker, r.company_name, r.space_revenue_pct as i32
            );
        }
    }

    Ok(())
}

/// Update fundamental data for all companies
pub async fn update_fundamentals(_concurrency: usize) -> Result<()> {
    info!("Updating fundamental data for all companies");

    println!("[!] This feature requires database integration");
    println!("   Will be implemented after database setup");

    Ok(())
}
