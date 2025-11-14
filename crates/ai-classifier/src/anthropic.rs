// Anthropic Claude API client for company classification (ported from Python)

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const MODEL: &str = "claude-3-haiku-20240307";

#[derive(Clone)]
pub struct AnthropicClassifier {
    api_key: String,
    client: Client,
    model: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Classification {
    pub ticker: String,
    pub company_name: String,
    pub is_space_related: bool,
    pub space_revenue_pct: f32,
    pub confidence: String,
    pub segments: Vec<String>,
    pub reasoning: String,
    #[serde(skip)]
    pub raw_response: String,
}

// Anthropic API request/response types
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    temperature: f32,
    messages: Vec<Message>,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<Content>,
}

#[derive(Debug, Deserialize)]
struct Content {
    text: String,
}

// Classification result from AI (matches JSON format)
#[derive(Debug, Deserialize)]
struct ClassificationData {
    is_space_related: bool,
    space_revenue_pct: f32,
    confidence: String,
    segments: Vec<String>,
    reasoning: String,
}

impl AnthropicClassifier {
    /// Create new classifier with API key from environment or parameter
    pub fn new(api_key: Option<String>) -> Result<Self> {
        let api_key = api_key
            .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
            .context("ANTHROPIC_API_KEY must be set in environment or passed to constructor")?;

        Ok(Self {
            api_key,
            client: Client::new(),
            model: MODEL.to_string(),
        })
    }

    /// Classify a company as space-related and estimate space revenue percentage
    pub async fn classify_company(
        &self,
        ticker: &str,
        company_name: &str,
        description: &str,
        additional_context: Option<&str>,
    ) -> Result<Classification> {
        let prompt = self.build_classification_prompt(
            ticker,
            company_name,
            description,
            additional_context,
        );

        debug!("Classifying company: {} ({})", company_name, ticker);

        let request = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: 2000,
            temperature: 0.0, // Deterministic
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt,
            }],
        };

        let response = self
            .client
            .post(ANTHROPIC_API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Anthropic API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Anthropic API error {}: {}", status, error_text);
        }

        let api_response: AnthropicResponse = response
            .json()
            .await
            .context("Failed to parse Anthropic API response")?;

        let response_text = &api_response.content[0].text;

        self.parse_response(ticker, company_name, response_text)
    }

    /// Build classification prompt for Claude
    fn build_classification_prompt(
        &self,
        ticker: &str,
        company_name: &str,
        description: &str,
        additional_context: Option<&str>,
    ) -> String {
        let mut prompt = format!(
            r#"You are an expert analyst specializing in the space infrastructure industry. Your task is to analyze companies and determine their involvement in space infrastructure.

Company Information:
- Ticker: {}
- Name: {}
- Description: {}
"#,
            ticker, company_name, description
        );

        if let Some(context) = additional_context {
            prompt.push_str(&format!("\nAdditional Context:\n{}\n", context));
        }

        prompt.push_str(
            r#"

Space Infrastructure Segments:
1. **Launch**: Launch vehicles, launch services, spaceports
2. **Satellites**: Satellite manufacturing, satellite operators, constellation services
3. **Ground**: Ground stations, tracking systems, antenna systems, data centers
4. **Components**: Propulsion systems, sensors, materials, avionics, spacecraft components

Your Analysis Task:
Analyze this company and provide your assessment in the following JSON format:

{
  "is_space_related": true/false,
  "space_revenue_pct": <number 0-100>,
  "confidence": "high/medium/low",
  "segments": [<list of applicable segments from above>],
  "reasoning": "<brief explanation of your assessment>"
}

Guidelines:
- is_space_related: true if ANY meaningful portion of business involves space infrastructure
- space_revenue_pct: Your estimate of what % of total revenue comes from space activities
  - 100% = Pure-play space company (e.g., Rocket Lab, AST SpaceMobile)
  - 50-99% = Primarily space with some other business
  - 10-49% = Significant space division within larger company
  - 1-9% = Minor space involvement
  - 0% = No space involvement
- confidence:
  - "high" = Clear space focus, good information available
  - "medium" = Some space involvement but uncertain extent
  - "low" = Limited information or ambiguous business model
- segments: List ALL applicable segments (can be multiple)
- reasoning: 2-3 sentences explaining your assessment and space_revenue_pct estimate

Important: Be conservative with space_revenue_pct estimates. Only assign high percentages (>50%) for clear pure-play or space-focused companies.

Return ONLY the JSON object, no other text.
"#,
        );

        prompt
    }

    /// Parse Claude's JSON response into Classification
    fn parse_response(
        &self,
        ticker: &str,
        company_name: &str,
        response_text: &str,
    ) -> Result<Classification> {
        // Extract JSON from response (in case there's extra text)
        let start_idx = response_text.find('{');
        let end_idx = response_text.rfind('}');

        match (start_idx, end_idx) {
            (Some(start), Some(end)) if start < end => {
                let json_str = &response_text[start..=end];

                match serde_json::from_str::<ClassificationData>(json_str) {
                    Ok(data) => Ok(Classification {
                        ticker: ticker.to_string(),
                        company_name: company_name.to_string(),
                        is_space_related: data.is_space_related,
                        space_revenue_pct: data.space_revenue_pct,
                        confidence: data.confidence,
                        segments: data.segments,
                        reasoning: data.reasoning,
                        raw_response: response_text.to_string(),
                    }),
                    Err(e) => {
                        warn!("Error parsing AI response: {}", e);
                        warn!("Response: {}", response_text);

                        Ok(Classification {
                            ticker: ticker.to_string(),
                            company_name: company_name.to_string(),
                            is_space_related: false,
                            space_revenue_pct: 0.0,
                            confidence: "low".to_string(),
                            segments: vec![],
                            reasoning: format!("Error parsing AI response: {}", e),
                            raw_response: response_text.to_string(),
                        })
                    }
                }
            }
            _ => {
                warn!("No JSON found in response: {}", response_text);

                Ok(Classification {
                    ticker: ticker.to_string(),
                    company_name: company_name.to_string(),
                    is_space_related: false,
                    space_revenue_pct: 0.0,
                    confidence: "low".to_string(),
                    segments: vec![],
                    reasoning: "No JSON found in AI response".to_string(),
                    raw_response: response_text.to_string(),
                })
            }
        }
    }

    /// Classify multiple companies in batch
    pub async fn batch_classify(
        &self,
        companies: Vec<CompanyInfo>,
        verbose: bool,
    ) -> Vec<Classification> {
        let mut results = Vec::new();
        let total = companies.len();

        for (i, company) in companies.into_iter().enumerate() {
            if verbose {
                println!(
                    "Classifying {}/{}: {} - {}",
                    i + 1,
                    total,
                    company.ticker,
                    company.name
                );
            }

            match self
                .classify_company(
                    &company.ticker,
                    &company.name,
                    &company.description,
                    company.context.as_deref(),
                )
                .await
            {
                Ok(result) => {
                    if verbose {
                        println!(
                            "  → Space: {}, Revenue %: {:.0}%, Segments: {}",
                            result.is_space_related,
                            result.space_revenue_pct,
                            result.segments.join(", ")
                        );
                    }
                    results.push(result);
                }
                Err(e) => {
                    warn!("Error classifying {}: {}", company.ticker, e);
                    results.push(Classification {
                        ticker: company.ticker.clone(),
                        company_name: company.name.clone(),
                        is_space_related: false,
                        space_revenue_pct: 0.0,
                        confidence: "low".to_string(),
                        segments: vec![],
                        reasoning: format!("Error: {}", e),
                        raw_response: String::new(),
                    });
                }
            }
        }

        results
    }
}

#[derive(Debug, Clone)]
pub struct CompanyInfo {
    pub ticker: String,
    pub name: String,
    pub description: String,
    pub context: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_building() {
        let classifier = AnthropicClassifier {
            api_key: "test_key".to_string(),
            client: Client::new(),
            model: MODEL.to_string(),
        };

        let prompt = classifier.build_classification_prompt(
            "RKLB",
            "Rocket Lab",
            "Space launch provider",
            None,
        );

        assert!(prompt.contains("RKLB"));
        assert!(prompt.contains("Rocket Lab"));
        assert!(prompt.contains("Space launch provider"));
        assert!(prompt.contains("Space Infrastructure Segments"));
    }

    #[test]
    fn test_json_parsing() {
        let classifier = AnthropicClassifier {
            api_key: "test_key".to_string(),
            client: Client::new(),
            model: MODEL.to_string(),
        };

        let response = r#"{
            "is_space_related": true,
            "space_revenue_pct": 90.0,
            "confidence": "high",
            "segments": ["Launch", "Satellites"],
            "reasoning": "Rocket Lab is a pure-play space company."
        }"#;

        let result = classifier
            .parse_response("RKLB", "Rocket Lab", response)
            .unwrap();

        assert!(result.is_space_related);
        assert_eq!(result.space_revenue_pct, 90.0);
        assert_eq!(result.confidence, "high");
        assert_eq!(result.segments.len(), 2);
    }
}
