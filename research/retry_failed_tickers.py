"""
Retry failed tickers from universe expansion.

This script:
1. Loads existing classified companies
2. Retries the failed tickers
3. Merges results
4. Recalculates index weights
"""

import pandas as pd
import numpy as np
import time
from polygon_client import PolygonClient
from ai_classifier import SpaceCompanyClassifier
from weighting_algorithm import SpaceIndexWeighting

# Initialize clients
polygon_client = PolygonClient()
ai_classifier = SpaceCompanyClassifier()

print("SPACEINFRA Universe Expansion - Retry Failed Tickers")
print("="*80 + "\n")

# Load existing classified companies
try:
    df_existing = pd.read_csv('classified_space_companies.csv')
    print(f"Loaded {len(df_existing)} existing space companies")
    existing_tickers = set(df_existing['ticker'].tolist())
except FileNotFoundError:
    print("No existing classifications found, starting fresh")
    df_existing = pd.DataFrame()
    existing_tickers = set()

# Failed tickers from previous run (high-priority ones)
failed_tickers = [
    # High priority - pure space plays we're missing
    "RKLB",   # Rocket Lab - OUR TOP PERFORMER!
    "LUNR",   # Intuitive Machines
    "VSAT",   # Viasat
    "PL",     # Planet Labs (if failed)

    # Aerospace/defense with space divisions
    "BA",     # Boeing
    "NOC",    # Northrop Grumman
    "RTX",    # Raytheon
    "HII",    # Huntington Ingalls

    # Satellite/telecom
    "DISH",   # Dish Network
    "CMCSA",  # Comcast

    # Other candidates
    "AJRD",   # Aerojet Rocketdyne
    "BLDE",   # Blade Air Mobility
    "JOBY",   # Joby Aviation
    "AVAV",   # AeroVironment
    "MNTS",   # Momentus
    "SATS",   # EchoStar
    "AXON",   # Axon (likely not space, but checking)
    "HON",    # Honeywell
    "GILT",   # Gilat
    "MKSI",   # MKS Instruments
    "MAXR",   # Maxar
    "VORB",   # Virgin Orbit
]

# Remove tickers we already have
failed_tickers = [t for t in failed_tickers if t not in existing_tickers]

print(f"Retrying {len(failed_tickers)} failed tickers")
print(f"Already classified: {len(existing_tickers)} tickers\n")

# Add a small initial delay to respect rate limits
print("Waiting 10 seconds before starting...")
time.sleep(10)

print("\n" + "="*80)
print("PHASE 1: Retry Failed Tickers")
print("="*80 + "\n")

newly_classified = []
still_failed = []

for i, ticker in enumerate(failed_tickers, 1):
    print(f"[{i}/{len(failed_tickers)}] {ticker}")
    print("-" * 80)

    try:
        # Get company details
        details = polygon_client.get_ticker_details(ticker)
        result = details.get('results', {})

        if not result:
            print(f"  ✗ No data found")
            still_failed.append({'ticker': ticker, 'error': 'No data found'})
            time.sleep(2)
            continue

        name = result.get('name', '')
        description = result.get('description', '')
        market_cap = result.get('market_cap', 0)

        print(f"  Name: {name}")
        print(f"  Market Cap: ${market_cap:,.0f}")

        # Classify with AI
        classification = ai_classifier.classify_company(
            ticker=ticker,
            company_name=name,
            description=description
        )

        print(f"  🤖 Space Related: {classification.is_space_related}")
        print(f"  🤖 Space Revenue %: {classification.space_revenue_pct:.0f}%")
        print(f"  🤖 Confidence: {classification.confidence}")
        print(f"  🤖 Segments: {', '.join(classification.segments)}")

        # Store if space-related
        if classification.is_space_related and classification.space_revenue_pct > 0:
            newly_classified.append({
                'ticker': ticker,
                'name': name,
                'market_cap': market_cap,
                'space_revenue_pct': classification.space_revenue_pct,
                'confidence': classification.confidence,
                'segments': ', '.join(classification.segments),
                'reasoning': classification.reasoning
            })
            print(f"  ✓ Added to universe")
        else:
            print(f"  ✗ Not space-related")

        # Conservative rate limiting - 3 seconds between requests
        time.sleep(3)

    except Exception as e:
        print(f"  ✗ Error: {e}")
        still_failed.append({'ticker': ticker, 'error': str(e)})
        time.sleep(3)
        continue

print("\n" + "="*80)
print(f"PHASE 1 COMPLETE: Found {len(newly_classified)} additional space companies")
print("="*80 + "\n")

# Merge with existing
if newly_classified:
    df_new = pd.DataFrame(newly_classified)

    if not df_existing.empty:
        df_all_classified = pd.concat([df_existing, df_new], ignore_index=True)
    else:
        df_all_classified = df_new

    # Remove duplicates
    df_all_classified = df_all_classified.drop_duplicates(subset=['ticker'])

    # Sort by space revenue %
    df_all_classified = df_all_classified.sort_values('space_revenue_pct', ascending=False)

    print(f"Total space companies: {len(df_all_classified)}")
    print("\nAll classified companies:")
    print(df_all_classified[['ticker', 'name', 'space_revenue_pct', 'confidence']].to_string(index=False))

    # Save updated classification
    df_all_classified.to_csv('classified_space_companies_updated.csv', index=False)
    print(f"\n✓ Saved to classified_space_companies_updated.csv")

else:
    print("No new companies found")
    df_all_classified = df_existing

if still_failed:
    print(f"\n⚠️ Still failed for {len(still_failed)} tickers:")
    for err in still_failed[:10]:
        print(f"  {err['ticker']}: {err['error']}")

# If we have enough companies, proceed to Phase 2
if len(df_all_classified) == 0:
    print("\n✗ No space companies to build index with")
    exit(1)

print("\n" + "="*80)
print("PHASE 2: Fetch Financial Data")
print("="*80 + "\n")

# Fetch revenue growth for companies we don't have yet
growth_data = []

for idx, row in df_all_classified.iterrows():
    ticker = row['ticker']

    # Skip if we already have this data
    if ticker in existing_tickers:
        print(f"{ticker}: Using existing data")
        continue

    print(f"Fetching financials for {ticker}...")

    try:
        financials = polygon_client.get_financials(ticker, timeframe="annual", limit=3)

        if financials and len(financials) >= 2:
            revenues = []
            for period in financials:
                income_stmt = period.get('financials', {}).get('income_statement', {})
                revenue = income_stmt.get('revenues', {}).get('value')
                fiscal_year = period.get('fiscal_year')

                if revenue and fiscal_year:
                    revenues.append({'year': fiscal_year, 'revenue': revenue})

            revenues = sorted(revenues, key=lambda x: x['year'])

            if len(revenues) >= 2:
                latest = revenues[-1]['revenue']
                previous = revenues[-2]['revenue']
                growth_yoy = ((latest - previous) / previous) * 100 if previous != 0 else 0

                print(f"  YoY Growth: {growth_yoy:.1f}%")
                growth_data.append({'ticker': ticker, 'revenue_growth_rate': growth_yoy})
            else:
                print(f"  Using estimated growth: 20%")
                growth_data.append({'ticker': ticker, 'revenue_growth_rate': 20.0})
        else:
            print(f"  Using estimated growth: 20%")
            growth_data.append({'ticker': ticker, 'revenue_growth_rate': 20.0})

        time.sleep(2)

    except Exception as e:
        print(f"  Error: {e}, using estimated growth: 20%")
        growth_data.append({'ticker': ticker, 'revenue_growth_rate': 20.0})
        time.sleep(2)

# Load existing growth data if available
try:
    df_prev_index = pd.read_csv('spaceinfra_index_25_composition.csv')
    existing_growth = df_prev_index[['ticker', 'revenue_growth_rate']]

    # Merge new growth data
    df_growth_new = pd.DataFrame(growth_data)
    df_growth_combined = pd.concat([existing_growth, df_growth_new], ignore_index=True)
    df_growth_combined = df_growth_combined.drop_duplicates(subset=['ticker'])

except FileNotFoundError:
    df_growth_combined = pd.DataFrame(growth_data)

# Merge with classified companies
df_final = df_all_classified.merge(df_growth_combined, on='ticker', how='left')

# Fill missing growth rates with 20%
df_final['revenue_growth_rate'] = df_final['revenue_growth_rate'].fillna(20.0)

print("\n" + "="*80)
print("PHASE 3: Calculate Index Weights")
print("="*80 + "\n")

# Determine max position size based on number of companies
num_companies = len(df_final)
if num_companies >= 25:
    max_position = 0.08  # 8% for 25+ companies
    min_position = 0.01  # 1%
elif num_companies >= 15:
    max_position = 0.10  # 10% for 15-24 companies
    min_position = 0.01
else:
    max_position = 0.12  # 12% for <15 companies
    min_position = 0.01

print(f"Index parameters: {num_companies} companies, max position {max_position*100:.0f}%")

# Calculate weights
weighting = SpaceIndexWeighting(
    space_revenue_weight=0.4,
    market_cap_weight=0.3,
    growth_weight=0.3,
    max_position_size=max_position,
    min_position_size=min_position
)

constituents = weighting.calculate_weights(df_final)

# Display results
print(f"\nSPACEINFRA Index - Expanded Universe ({len(constituents)} companies)\n")
print(f"{'Rank':<6} {'Ticker':<8} {'Name':<35} {'Weight':<10} {'Space%':<10} {'Growth%':<10} {'Mkt Cap':<12}")
print("-" * 130)

for i, c in enumerate(constituents, 1):
    market_cap_b = c.market_cap / 1e9 if c.market_cap else 0
    name_short = c.name[:33] if len(c.name) > 33 else c.name
    print(f"{i:<6} {c.ticker:<8} {name_short:<35} {c.weight*100:>6.2f}%   "
          f"{c.space_revenue_pct:>6.1f}%   {c.revenue_growth_rate:>7.1f}%   "
          f"${market_cap_b:>6.2f}B")

# Summary statistics
print("\n" + "="*80)
print("Index Statistics:")
print("="*80 + "\n")

stats = weighting.summary_stats(constituents)
total_market_cap = sum(c.market_cap for c in constituents if c.market_cap)

print(f"Number of Constituents: {stats['num_constituents']}")
print(f"Total Market Cap: ${total_market_cap/1e9:.2f}B")
print(f"\nWeighted Averages:")
print(f"  Space Revenue %: {stats['weighted_avg_space_rev_pct']:.1f}%")
print(f"  Revenue Growth: {stats['weighted_avg_growth']:.1f}%")
print(f"\nPosition Sizes:")
print(f"  Largest Position: {stats['max_weight']*100:.1f}%")
print(f"  Smallest Position: {stats['min_weight']*100:.1f}%")

# Segment breakdown
segments_list = []
for c in constituents:
    if c.segments:
        for seg in c.segments.split(','):
            segments_list.append(seg.strip())

from collections import Counter
segment_counts = Counter(segments_list)

print(f"\nSegment Exposure:")
for segment, count in segment_counts.most_common():
    print(f"  {segment}: {count} companies")

# Save final index
df_index = pd.DataFrame([{
    'rank': i,
    'ticker': c.ticker,
    'name': c.name,
    'weight': c.weight,
    'market_cap': c.market_cap,
    'space_revenue_pct': c.space_revenue_pct,
    'revenue_growth_rate': c.revenue_growth_rate,
    'segments': c.segments,
} for i, c in enumerate(constituents, 1)])

df_index.to_csv('spaceinfra_index_final.csv', index=False)
print(f"\n✓ Final index saved to spaceinfra_index_final.csv")

# Comparison to previous
try:
    df_prev = pd.read_csv('spaceinfra_index_25_composition.csv')
    print(f"\n📊 Index Growth: {len(df_prev)} → {len(df_index)} companies (+{len(df_index) - len(df_prev)})")
except:
    pass

print("\n" + "="*80)
print("Universe Expansion Complete!")
print("="*80)
