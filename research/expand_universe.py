"""
Expand SPACEINFRA index to 25+ companies.

Process:
1. Create candidate list of aerospace/satellite/defense companies
2. Classify with AI to determine space exposure
3. Filter to space-related companies
4. Fetch financials
5. Apply weighting algorithm
6. Generate 25-company index
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

print("SPACEINFRA Universe Expansion")
print("="*80 + "\n")

# Candidate companies - aerospace, defense, satellite, telecom with space exposure
# Mix of pure-play space, aerospace/defense with space divisions, and satellite operators
candidate_tickers = [
    # Existing 5
    "ASTS", "RKLB", "SPCE", "IRDM", "GSAT",

    # Pure-play space companies
    "LUNR",   # Intuitive Machines - lunar landers
    "PL",     # Planet Labs - Earth observation
    "SPIR",   # Spire Global - satellite data
    "SATS",   # EchoStar / Dish satellites (if available)
    "VSAT",   # Viasat - satellite broadband
    "GILT",   # Gilat Satellite Networks
    "IRDM",   # Already have, but keeping for completeness

    # Aerospace/Defense with significant space divisions
    "LMT",    # Lockheed Martin - satellites, space systems
    "BA",     # Boeing - space launch, satellites
    "NOC",    # Northrop Grumman - space systems
    "RTX",    # Raytheon - space and missile defense
    "LHX",    # L3Harris - space systems
    "TDY",    # Teledyne - space imaging, sensors
    "HII",    # Huntington Ingalls - space systems division
    "AJRD",   # Aerojet Rocketdyne - propulsion (if still public)
    "TXT",    # Textron - aerospace systems

    # Satellite operators and telecom
    "DISH",   # Dish Network - satellite TV + spectrum
    "CMCSA",  # Comcast - satellite infrastructure
    "SIRI",   # SiriusXM - satellite radio

    # Space components and technology
    "KTOS",   # Kratos Defense - satellite communications, space tech
    "AVAV",   # AeroVironment - space-related drones/systems
    "MKSI",   # MKS Instruments - components for space manufacturing
    "HON",    # Honeywell - aerospace and space systems
    "AXON",   # Axon Enterprise - (unlikely, but checking)

    # Emerging space SPACs and recent IPOs
    "ACHR",   # Archer Aviation - electric aircraft (space-adjacent)
    "JOBY",   # Joby Aviation - electric aircraft (space-adjacent)
    "BLDE",   # Blade Air Mobility

    # International space (US-traded)
    "MAXR",   # Maxar Technologies - Earth observation, satellites (if available)

    # Small cap space plays
    "VORB",   # Virgin Orbit (if still trading)
    "MNTS",   # Momentus - space infrastructure services (if available)
]

print(f"Candidate universe: {len(candidate_tickers)} companies")
print("Removing duplicates...")

# Remove duplicates
candidate_tickers = list(set(candidate_tickers))
print(f"After deduplication: {len(candidate_tickers)} companies\n")

print("="*80)
print("PHASE 1: Fetch Company Data and Classify")
print("="*80 + "\n")

classified_companies = []
errors = []

for i, ticker in enumerate(candidate_tickers, 1):
    print(f"[{i}/{len(candidate_tickers)}] {ticker}")
    print("-" * 80)

    try:
        # Get company details
        details = polygon_client.get_ticker_details(ticker)
        result = details.get('results', {})

        if not result:
            print(f"  ✗ No data found")
            errors.append({'ticker': ticker, 'error': 'No data found'})
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
            classified_companies.append({
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

        # Rate limiting - be conservative
        time.sleep(2)

    except Exception as e:
        print(f"  ✗ Error: {e}")
        errors.append({'ticker': ticker, 'error': str(e)})
        time.sleep(2)
        continue

print("\n" + "="*80)
print(f"PHASE 1 COMPLETE: Found {len(classified_companies)} space-related companies")
print("="*80 + "\n")

# Convert to DataFrame
df_classified = pd.DataFrame(classified_companies)

if len(df_classified) == 0:
    print("✗ No space companies found. Exiting.")
    exit(1)

# Sort by space revenue %
df_classified = df_classified.sort_values('space_revenue_pct', ascending=False)

print("Space-related companies found:")
print(df_classified[['ticker', 'name', 'space_revenue_pct', 'confidence', 'segments']].to_string(index=False))

# Save intermediate results
df_classified.to_csv('classified_space_companies.csv', index=False)
print(f"\n✓ Saved to classified_space_companies.csv")

# If we have more than 25, we're good. Otherwise report how many we have
target_count = 25
if len(df_classified) >= target_count:
    print(f"\n✓ Success! Found {len(df_classified)} companies (target: {target_count})")
    df_to_use = df_classified.head(target_count)
else:
    print(f"\n⚠️ Found {len(df_classified)} companies (target: {target_count})")
    print(f"  Using all {len(df_classified)} companies")
    df_to_use = df_classified

print("\n" + "="*80)
print("PHASE 2: Fetch Financial Data")
print("="*80 + "\n")

# Fetch revenue growth data
growth_data = []

for idx, row in df_to_use.iterrows():
    ticker = row['ticker']
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

        time.sleep(1.5)

    except Exception as e:
        print(f"  Error: {e}, using estimated growth: 20%")
        growth_data.append({'ticker': ticker, 'revenue_growth_rate': 20.0})
        time.sleep(1.5)

# Merge growth data
df_growth = pd.DataFrame(growth_data)
df_final = df_to_use.merge(df_growth, on='ticker')

print("\n" + "="*80)
print("PHASE 3: Calculate Index Weights")
print("="*80 + "\n")

# Calculate weights
weighting = SpaceIndexWeighting(
    space_revenue_weight=0.4,
    market_cap_weight=0.3,
    growth_weight=0.3,
    max_position_size=0.10,  # Reduce to 10% max for larger index
    min_position_size=0.01   # 1% minimum
)

constituents = weighting.calculate_weights(df_final)

# Display results
print(f"SPACEINFRA Index - Expanded Universe ({len(constituents)} companies)\n")
print(f"{'Rank':<6} {'Ticker':<8} {'Name':<30} {'Weight':<10} {'Space%':<10} {'Growth%':<10} {'Mkt Cap':<12}")
print("-" * 120)

for i, c in enumerate(constituents, 1):
    market_cap_b = c.market_cap / 1e9 if c.market_cap else 0
    print(f"{i:<6} {c.ticker:<8} {c.name[:28]:<30} {c.weight*100:>6.2f}%   "
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

df_index.to_csv('spaceinfra_index_25_composition.csv', index=False)
print(f"\n✓ Index saved to spaceinfra_index_25_composition.csv")

# Error summary
if errors:
    print(f"\n⚠️ Errors encountered for {len(errors)} tickers:")
    for err in errors[:10]:  # Show first 10
        print(f"  {err['ticker']}: {err['error']}")

print("\n" + "="*80)
print("Universe Expansion Complete!")
print("="*80)
