"""
Build initial SPACEINFRA index with real data.

Fetches financial data from Polygon.io and applies weighting algorithm.
"""

import pandas as pd
import time
from polygon_client import PolygonClient
from weighting_algorithm import SpaceIndexWeighting

# Initialize client
polygon_client = PolygonClient()

# Our classified space companies from batch test
classified_companies = pd.DataFrame([
    {
        'ticker': 'ASTS',
        'name': 'AST SpaceMobile',
        'market_cap': 19.188e9,
        'space_revenue_pct': 90,
        'segments': 'Satellites, Ground'
    },
    {
        'ticker': 'RKLB',
        'name': 'Rocket Lab',
        'market_cap': 24.992e9,
        'space_revenue_pct': 80,
        'segments': 'Launch, Satellites'
    },
    {
        'ticker': 'SPCE',
        'name': 'Virgin Galactic',
        'market_cap': 0.207e9,
        'space_revenue_pct': 50,
        'segments': 'Launch, Ground'
    },
    {
        'ticker': 'IRDM',
        'name': 'Iridium',
        'market_cap': 1.758e9,
        'space_revenue_pct': 50,
        'segments': 'Satellites, Ground'
    },
    {
        'ticker': 'GSAT',
        'name': 'Globalstar',
        'market_cap': 6.403e9,
        'space_revenue_pct': 30,
        'segments': 'Satellites, Ground'
    },
])

print("SPACEINFRA Index Construction")
print("="*80 + "\n")
print(f"Fetching financial data for {len(classified_companies)} companies...\n")

# Fetch revenue and calculate growth rates
growth_data = []

for idx, row in classified_companies.iterrows():
    ticker = row['ticker']
    print(f"Fetching financials for {ticker}...")

    try:
        # Get annual financials
        financials = polygon_client.get_financials(ticker, timeframe="annual", limit=3)

        if financials and len(financials) >= 2:
            # Extract revenues
            revenues = []
            for period in financials:
                income_stmt = period.get('financials', {}).get('income_statement', {})
                revenue = income_stmt.get('revenues', {}).get('value')
                fiscal_year = period.get('fiscal_year')

                if revenue and fiscal_year:
                    revenues.append({
                        'year': fiscal_year,
                        'revenue': revenue
                    })

            # Sort by year
            revenues = sorted(revenues, key=lambda x: x['year'])

            if len(revenues) >= 2:
                # Calculate YoY growth
                latest = revenues[-1]['revenue']
                previous = revenues[-2]['revenue']
                growth_yoy = ((latest - previous) / previous) * 100 if previous != 0 else 0

                print(f"  Latest Revenue: ${latest:,.0f}")
                print(f"  Previous Revenue: ${previous:,.0f}")
                print(f"  YoY Growth: {growth_yoy:.1f}%")

                growth_data.append({
                    'ticker': ticker,
                    'revenue_growth_rate': growth_yoy
                })
            else:
                print(f"  Not enough revenue data, using estimate")
                # Use estimated growth based on company stage
                estimated_growth = {
                    'ASTS': 150,  # Pre-revenue/early revenue, high growth
                    'RKLB': 50,   # Growing launch provider
                    'SPCE': -20,  # Struggling
                    'IRDM': 5,    # Mature, stable
                    'GSAT': 15,   # Moderate growth
                }.get(ticker, 10)

                growth_data.append({
                    'ticker': ticker,
                    'revenue_growth_rate': estimated_growth
                })
        else:
            print(f"  No financial data available, using estimate")
            estimated_growth = {
                'ASTS': 150,
                'RKLB': 50,
                'SPCE': -20,
                'IRDM': 5,
                'GSAT': 15,
            }.get(ticker, 10)

            growth_data.append({
                'ticker': ticker,
                'revenue_growth_rate': estimated_growth
            })

        time.sleep(1)  # Rate limiting

    except Exception as e:
        print(f"  Error: {e}")
        print(f"  Using estimated growth")

        estimated_growth = {
            'ASTS': 150,
            'RKLB': 50,
            'SPCE': -20,
            'IRDM': 5,
            'GSAT': 15,
        }.get(ticker, 10)

        growth_data.append({
            'ticker': ticker,
            'revenue_growth_rate': estimated_growth
        })

# Merge growth data
df_growth = pd.DataFrame(growth_data)
df_companies = classified_companies.merge(df_growth, on='ticker')

print("\n" + "="*80)
print("Company Data Summary:")
print("="*80 + "\n")
print(df_companies[['ticker', 'name', 'market_cap', 'space_revenue_pct', 'revenue_growth_rate']].to_string(index=False))

# Calculate weights
print("\n" + "="*80)
print("Calculating Index Weights...")
print("="*80 + "\n")

weighting = SpaceIndexWeighting(
    space_revenue_weight=0.4,
    market_cap_weight=0.3,
    growth_weight=0.3,
    max_position_size=0.15,
    min_position_size=0.01
)

constituents = weighting.calculate_weights(df_companies)

# Display index composition
print("SPACEINFRA Index Composition:\n")
print(f"{'Rank':<6} {'Ticker':<8} {'Name':<25} {'Weight':<10} {'Mkt Cap':<12} {'Space%':<10} {'Growth%':<10} {'Segments':<30}")
print("-" * 130)

for i, c in enumerate(constituents, 1):
    market_cap_b = c.market_cap / 1e9
    print(f"{i:<6} {c.ticker:<8} {c.name:<25} {c.weight*100:>6.2f}%   "
          f"${market_cap_b:>6.2f}B   {c.space_revenue_pct:>6.1f}%   "
          f"{c.revenue_growth_rate:>7.1f}%   {c.segments:<30}")

# Summary statistics
print("\n" + "="*80)
print("Index Statistics:")
print("="*80 + "\n")

stats = weighting.summary_stats(constituents)
print(f"Number of Constituents: {stats['num_constituents']}")
print(f"Total Index Weight: {stats['total_weight']*100:.1f}%")
print(f"Total Market Cap: ${sum(c.market_cap for c in constituents)/1e9:.2f}B")
print(f"\nWeighted Averages:")
print(f"  Space Revenue %: {stats['weighted_avg_space_rev_pct']:.1f}%")
print(f"  Revenue Growth: {stats['weighted_avg_growth']:.1f}%")
print(f"\nPosition Sizes:")
print(f"  Largest Position: {stats['max_weight']*100:.1f}%")
print(f"  Smallest Position: {stats['min_weight']*100:.1f}%")

# Save to CSV
df_index = pd.DataFrame([{
    'ticker': c.ticker,
    'name': c.name,
    'weight': c.weight,
    'market_cap': c.market_cap,
    'space_revenue_pct': c.space_revenue_pct,
    'revenue_growth_rate': c.revenue_growth_rate,
    'segments': c.segments,
    'raw_score': c.raw_score
} for c in constituents])

df_index.to_csv('spaceinfra_index_composition.csv', index=False)
print(f"\n✓ Index composition saved to spaceinfra_index_composition.csv")
