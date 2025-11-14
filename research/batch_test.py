"""Quick batch test of AI classifier on space companies."""

import time
import pandas as pd
from polygon_client import PolygonClient
from ai_classifier import SpaceCompanyClassifier

# Initialize clients
print("Initializing clients...")
polygon_client = PolygonClient()
ai_classifier = SpaceCompanyClassifier()

# List of companies to test
companies_to_test = [
    # Known pure-play space companies
    "ASTS",   # AST SpaceMobile - satellite connectivity
    "RKLB",   # Rocket Lab - launch provider
    "SPCE",   # Virgin Galactic - space tourism/launch
    "GSAT",   # Globalstar - satellite communications
    "IRDM",   # Iridium - satellite communications
    "LUNR",   # Intuitive Machines - lunar infrastructure
    "PL",     # Planet Labs - Earth observation

    # Aerospace/defense with space divisions
    "BA",     # Boeing
    "LMT",    # Lockheed Martin

    # Non-space (validation)
    "AAPL",   # Apple
]

print(f"\nTesting {len(companies_to_test)} companies...\n")
print("="*80)

results = []

for i, ticker in enumerate(companies_to_test, 1):
    try:
        print(f"\n[{i}/{len(companies_to_test)}] {ticker}")
        print("-" * 80)

        # Get company details from Polygon
        details = polygon_client.get_ticker_details(ticker)
        result = details.get('results', {})

        name = result.get('name', 'N/A')
        description = result.get('description', '')
        market_cap = result.get('market_cap', 0)

        print(f"Name: {name}")
        print(f"Market Cap: ${market_cap:,.0f}")

        # Classify with AI
        classification = ai_classifier.classify_company(
            ticker=ticker,
            company_name=name,
            description=description
        )

        print(f"\n🤖 AI Classification:")
        print(f"  Space Related: {'✓ YES' if classification.is_space_related else '✗ NO'}")
        print(f"  Space Revenue %: {classification.space_revenue_pct:.0f}%")
        print(f"  Confidence: {classification.confidence.upper()}")
        print(f"  Segments: {', '.join(classification.segments) if classification.segments else 'None'}")
        print(f"  Reasoning: {classification.reasoning}")

        results.append({
            'ticker': ticker,
            'name': name,
            'market_cap': market_cap,
            'market_cap_b': market_cap / 1e9 if market_cap else 0,
            'is_space': classification.is_space_related,
            'space_revenue_pct': classification.space_revenue_pct,
            'confidence': classification.confidence,
            'segments': ', '.join(classification.segments),
            'reasoning': classification.reasoning
        })

        # Rate limiting
        if i < len(companies_to_test):
            time.sleep(2)

    except Exception as e:
        print(f"✗ Error processing {ticker}: {e}")
        results.append({
            'ticker': ticker,
            'name': 'Error',
            'market_cap': 0,
            'market_cap_b': 0,
            'is_space': False,
            'space_revenue_pct': 0,
            'confidence': 'error',
            'segments': '',
            'reasoning': str(e)
        })

# Create DataFrame and display summary
print("\n\n" + "="*80)
print("SUMMARY RESULTS")
print("="*80 + "\n")

df = pd.DataFrame(results)

# Sort by space revenue %
df_sorted = df.sort_values('space_revenue_pct', ascending=False)

print(df_sorted[['ticker', 'name', 'market_cap_b', 'space_revenue_pct', 'confidence', 'segments']].to_string(index=False))

# Statistics
print("\n" + "="*80)
print("STATISTICS")
print("="*80)
print(f"Total companies tested: {len(df)}")
print(f"Space-related companies: {df['is_space'].sum()}")
print(f"Average space revenue % (space companies only): {df[df['is_space']]['space_revenue_pct'].mean():.1f}%")
print(f"High confidence classifications: {(df['confidence'] == 'high').sum()}")

# Save results
df.to_csv('batch_test_results.csv', index=False)
print(f"\n✓ Results saved to batch_test_results.csv")
