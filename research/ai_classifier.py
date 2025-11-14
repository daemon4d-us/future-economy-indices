"""
AI-powered company classifier for identifying space infrastructure companies.

Uses Anthropic Claude to:
1. Determine if a company is space-related
2. Estimate % of revenue from space activities
3. Classify into space segments (launch, satellites, ground, components)
"""

import os
import json
from typing import Dict, List, Optional
from dataclasses import dataclass
from dotenv import load_dotenv
from anthropic import Anthropic

# Load environment variables
load_dotenv()


@dataclass
class SpaceClassification:
    """Result of space company classification."""
    ticker: str
    company_name: str
    is_space_related: bool
    space_revenue_pct: float  # 0-100
    confidence: str  # "high", "medium", "low"
    segments: List[str]  # ["launch", "satellites", "ground", "components"]
    reasoning: str
    raw_response: str


class SpaceCompanyClassifier:
    """Classifier for identifying and scoring space infrastructure companies."""

    def __init__(self, api_key: Optional[str] = None):
        """
        Initialize classifier with Anthropic API.

        Args:
            api_key: Anthropic API key. If not provided, reads from ANTHROPIC_API_KEY env var.
        """
        self.api_key = api_key or os.getenv("ANTHROPIC_API_KEY")
        if not self.api_key:
            raise ValueError("ANTHROPIC_API_KEY must be set in environment or passed to constructor")

        self.client = Anthropic(api_key=self.api_key)
        self.model = "claude-3-haiku-20240307"  # Claude 3 Haiku (fast, cost-effective)

    def classify_company(
        self,
        ticker: str,
        company_name: str,
        description: str,
        additional_context: Optional[str] = None
    ) -> SpaceClassification:
        """
        Classify a company as space-related and estimate space revenue percentage.

        Args:
            ticker: Stock ticker symbol
            company_name: Company name
            description: Company description or business summary
            additional_context: Optional additional info (news, filings, etc.)

        Returns:
            SpaceClassification with results
        """
        prompt = self._build_classification_prompt(
            ticker, company_name, description, additional_context
        )

        response = self.client.messages.create(
            model=self.model,
            max_tokens=2000,
            temperature=0.0,  # Deterministic for consistency
            messages=[{"role": "user", "content": prompt}]
        )

        # Parse the response
        response_text = response.content[0].text
        classification = self._parse_response(ticker, company_name, response_text)

        return classification

    def _build_classification_prompt(
        self,
        ticker: str,
        company_name: str,
        description: str,
        additional_context: Optional[str] = None
    ) -> str:
        """Build the classification prompt for Claude."""

        prompt = f"""You are an expert analyst specializing in the space infrastructure industry. Your task is to analyze companies and determine their involvement in space infrastructure.

Company Information:
- Ticker: {ticker}
- Name: {company_name}
- Description: {description}
"""

        if additional_context:
            prompt += f"\nAdditional Context:\n{additional_context}\n"

        prompt += """

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
"""
        return prompt

    def _parse_response(
        self,
        ticker: str,
        company_name: str,
        response_text: str
    ) -> SpaceClassification:
        """Parse Claude's JSON response into SpaceClassification object."""

        try:
            # Extract JSON from response (in case there's extra text)
            start_idx = response_text.find('{')
            end_idx = response_text.rfind('}') + 1
            json_str = response_text[start_idx:end_idx]

            data = json.loads(json_str)

            return SpaceClassification(
                ticker=ticker,
                company_name=company_name,
                is_space_related=data.get('is_space_related', False),
                space_revenue_pct=float(data.get('space_revenue_pct', 0)),
                confidence=data.get('confidence', 'low'),
                segments=data.get('segments', []),
                reasoning=data.get('reasoning', ''),
                raw_response=response_text
            )
        except (json.JSONDecodeError, ValueError) as e:
            # Fallback if parsing fails
            print(f"Error parsing response: {e}")
            print(f"Response: {response_text}")

            return SpaceClassification(
                ticker=ticker,
                company_name=company_name,
                is_space_related=False,
                space_revenue_pct=0.0,
                confidence='low',
                segments=[],
                reasoning=f"Error parsing AI response: {e}",
                raw_response=response_text
            )

    def batch_classify(
        self,
        companies: List[Dict[str, str]],
        verbose: bool = True
    ) -> List[SpaceClassification]:
        """
        Classify multiple companies in batch.

        Args:
            companies: List of dicts with keys: ticker, name, description
            verbose: Print progress

        Returns:
            List of SpaceClassification results
        """
        results = []

        for i, company in enumerate(companies):
            if verbose:
                print(f"Classifying {i+1}/{len(companies)}: {company['ticker']} - {company['name']}")

            try:
                result = self.classify_company(
                    ticker=company['ticker'],
                    company_name=company['name'],
                    description=company.get('description', ''),
                    additional_context=company.get('context')
                )
                results.append(result)

                if verbose:
                    print(f"  → Space: {result.is_space_related}, "
                          f"Revenue %: {result.space_revenue_pct:.0f}%, "
                          f"Segments: {', '.join(result.segments)}")

            except Exception as e:
                print(f"  ✗ Error: {e}")
                results.append(SpaceClassification(
                    ticker=company['ticker'],
                    company_name=company['name'],
                    is_space_related=False,
                    space_revenue_pct=0.0,
                    confidence='low',
                    segments=[],
                    reasoning=f"Error: {e}",
                    raw_response=""
                ))

        return results


def main():
    """Example usage of SpaceCompanyClassifier."""
    classifier = SpaceCompanyClassifier()

    # Test with a known space company
    print("Testing AI Classification System\n" + "="*50)

    test_company = {
        "ticker": "RKLB",
        "name": "Rocket Lab USA, Inc.",
        "description": """Rocket Lab is a global leader in launch services and space systems.
        The company's Electron launch vehicle provides dedicated small satellite launch services.
        Rocket Lab also manufactures spacecraft components and provides satellite manufacturing services.
        The company has launched over 150 satellites to orbit and operates launch sites in New Zealand and the United States."""
    }

    result = classifier.classify_company(
        ticker=test_company['ticker'],
        company_name=test_company['name'],
        description=test_company['description']
    )

    print(f"\nCompany: {result.company_name} ({result.ticker})")
    print(f"Space Related: {result.is_space_related}")
    print(f"Space Revenue %: {result.space_revenue_pct:.1f}%")
    print(f"Confidence: {result.confidence}")
    print(f"Segments: {', '.join(result.segments)}")
    print(f"\nReasoning: {result.reasoning}")


if __name__ == "__main__":
    main()
