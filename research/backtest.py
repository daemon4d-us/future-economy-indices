"""
Backtest SPACEINFRA index performance.

Calculates historical returns and compares to benchmarks.
"""

import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import seaborn as sns
from datetime import datetime, timedelta
from polygon_client import PolygonClient
import time

# Initialize client
polygon_client = PolygonClient()

# Load index composition
df_index = pd.read_csv('spaceinfra_index_composition.csv')

print("SPACEINFRA Index Backtesting")
print("="*80 + "\n")
print(f"Index constituents: {len(df_index)}")
print(df_index[['ticker', 'weight']].to_string(index=False))

# Set backtest period
end_date = datetime.now()
start_date = end_date - timedelta(days=365)  # 1 year backtest

print(f"\nBacktest period: {start_date.strftime('%Y-%m-%d')} to {end_date.strftime('%Y-%m-%d')}")
print("\nFetching historical price data...\n")

# Fetch historical data for each constituent
price_data = {}

for idx, row in df_index.iterrows():
    ticker = row['ticker']
    print(f"Fetching {ticker}...")

    try:
        bars = polygon_client.get_aggregates(
            ticker=ticker,
            from_date=start_date.strftime("%Y-%m-%d"),
            to_date=end_date.strftime("%Y-%m-%d")
        )

        if bars:
            df_bars = pd.DataFrame(bars)
            df_bars['date'] = pd.to_datetime(df_bars['t'], unit='ms')
            df_bars = df_bars.set_index('date')
            df_bars = df_bars[['c']].rename(columns={'c': ticker})
            price_data[ticker] = df_bars
            print(f"  ✓ Retrieved {len(df_bars)} days")
        else:
            print(f"  ✗ No data available")

    except Exception as e:
        print(f"  ✗ Error: {e}")

    time.sleep(1)  # Rate limiting

# Combine all price data
if price_data:
    df_prices = pd.concat(price_data.values(), axis=1)
    df_prices = df_prices.dropna()  # Only use dates where all constituents have data

    print(f"\n✓ Combined price data: {len(df_prices)} trading days")
    print(f"  Date range: {df_prices.index.min()} to {df_prices.index.max()}")
else:
    print("✗ No price data retrieved")
    exit(1)

# Calculate index returns
print("\nCalculating index performance...")

# Get weights as array
weights = df_index.set_index('ticker')['weight'].to_dict()
weight_array = np.array([weights.get(col, 0) for col in df_prices.columns])

# Calculate daily returns for each stock
returns = df_prices.pct_change()

# Calculate index returns (weighted average of constituent returns)
index_returns = (returns * weight_array).sum(axis=1)

# Calculate cumulative returns
cumulative_returns = (1 + index_returns).cumprod()
cumulative_returns.iloc[0] = 1.0  # Start at 1.0

# Fetch benchmark: SPY (S&P 500 ETF)
print("\nFetching benchmark data (SPY - S&P 500)...")

try:
    spy_bars = polygon_client.get_aggregates(
        ticker="SPY",
        from_date=start_date.strftime("%Y-%m-%d"),
        to_date=end_date.strftime("%Y-%m-%d")
    )

    if spy_bars:
        df_spy = pd.DataFrame(spy_bars)
        df_spy['date'] = pd.to_datetime(df_spy['t'], unit='ms')
        df_spy = df_spy.set_index('date')
        df_spy = df_spy[['c']].rename(columns={'c': 'SPY'})

        spy_returns = df_spy['SPY'].pct_change()
        spy_cumulative = (1 + spy_returns).cumprod()
        spy_cumulative.iloc[0] = 1.0

        print(f"  ✓ Retrieved {len(df_spy)} days")
    else:
        print("  ✗ No SPY data")
        spy_cumulative = None
except Exception as e:
    print(f"  ✗ Error fetching SPY: {e}")
    spy_cumulative = None

# Calculate performance metrics
print("\n" + "="*80)
print("Performance Metrics")
print("="*80 + "\n")

def calculate_metrics(returns, name="Index"):
    """Calculate performance metrics."""
    total_return = (1 + returns).prod() - 1
    annualized_return = (1 + total_return) ** (252 / len(returns)) - 1
    volatility = returns.std() * np.sqrt(252)
    sharpe_ratio = annualized_return / volatility if volatility != 0 else 0

    # Max drawdown
    cumulative = (1 + returns).cumprod()
    running_max = cumulative.expanding().max()
    drawdown = (cumulative - running_max) / running_max
    max_drawdown = drawdown.min()

    return {
        'name': name,
        'total_return': total_return * 100,
        'annualized_return': annualized_return * 100,
        'volatility': volatility * 100,
        'sharpe_ratio': sharpe_ratio,
        'max_drawdown': max_drawdown * 100,
        'best_day': returns.max() * 100,
        'worst_day': returns.min() * 100,
    }

# SPACEINFRA metrics
spaceinfra_metrics = calculate_metrics(index_returns, "SPACEINFRA")

print(f"SPACEINFRA Index:")
print(f"  Total Return: {spaceinfra_metrics['total_return']:>7.2f}%")
print(f"  Annualized Return: {spaceinfra_metrics['annualized_return']:>7.2f}%")
print(f"  Volatility (Ann.): {spaceinfra_metrics['volatility']:>7.2f}%")
print(f"  Sharpe Ratio: {spaceinfra_metrics['sharpe_ratio']:>10.2f}")
print(f"  Max Drawdown: {spaceinfra_metrics['max_drawdown']:>7.2f}%")
print(f"  Best Day: {spaceinfra_metrics['best_day']:>7.2f}%")
print(f"  Worst Day: {spaceinfra_metrics['worst_day']:>7.2f}%")

if spy_cumulative is not None:
    spy_metrics = calculate_metrics(spy_returns, "S&P 500")

    print(f"\nS&P 500 (SPY):")
    print(f"  Total Return: {spy_metrics['total_return']:>7.2f}%")
    print(f"  Annualized Return: {spy_metrics['annualized_return']:>7.2f}%")
    print(f"  Volatility (Ann.): {spy_metrics['volatility']:>7.2f}%")
    print(f"  Sharpe Ratio: {spy_metrics['sharpe_ratio']:>10.2f}")
    print(f"  Max Drawdown: {spy_metrics['max_drawdown']:>7.2f}%")

    print(f"\nOutperformance:")
    print(f"  Alpha: {spaceinfra_metrics['total_return'] - spy_metrics['total_return']:>7.2f}%")
    print(f"  Sharpe Difference: {spaceinfra_metrics['sharpe_ratio'] - spy_metrics['sharpe_ratio']:>7.2f}")

# Visualization
print("\nGenerating performance charts...")

fig = plt.figure(figsize=(16, 10))

# 1. Cumulative returns
ax1 = plt.subplot(2, 2, 1)
ax1.plot(cumulative_returns.index, (cumulative_returns - 1) * 100,
         linewidth=2, label='SPACEINFRA', color='#2E86AB')
if spy_cumulative is not None:
    ax1.plot(spy_cumulative.index, (spy_cumulative - 1) * 100,
             linewidth=2, label='S&P 500', color='#A23B72', alpha=0.7)
ax1.set_ylabel('Cumulative Return (%)', fontsize=12)
ax1.set_title('Cumulative Performance', fontsize=14, fontweight='bold')
ax1.legend(fontsize=11)
ax1.grid(True, alpha=0.3)
ax1.axhline(y=0, color='black', linestyle='-', linewidth=0.5)

# 2. Daily returns distribution
ax2 = plt.subplot(2, 2, 2)
ax2.hist(index_returns * 100, bins=50, alpha=0.7, color='#2E86AB', edgecolor='black')
ax2.axvline(x=0, color='red', linestyle='--', linewidth=1)
ax2.set_xlabel('Daily Return (%)', fontsize=12)
ax2.set_ylabel('Frequency', fontsize=12)
ax2.set_title('Daily Returns Distribution', fontsize=14, fontweight='bold')
ax2.grid(True, alpha=0.3)

# 3. Rolling volatility (30-day)
ax3 = plt.subplot(2, 2, 3)
rolling_vol = index_returns.rolling(30).std() * np.sqrt(252) * 100
ax3.plot(rolling_vol.index, rolling_vol, linewidth=2, color='#F18F01')
ax3.set_ylabel('Annualized Volatility (%)', fontsize=12)
ax3.set_title('30-Day Rolling Volatility', fontsize=14, fontweight='bold')
ax3.grid(True, alpha=0.3)

# 4. Drawdown
ax4 = plt.subplot(2, 2, 4)
running_max = cumulative_returns.expanding().max()
drawdown = (cumulative_returns - running_max) / running_max * 100
ax4.fill_between(drawdown.index, drawdown, 0, alpha=0.7, color='#C73E1D')
ax4.set_ylabel('Drawdown (%)', fontsize=12)
ax4.set_title('Drawdown from Peak', fontsize=14, fontweight='bold')
ax4.grid(True, alpha=0.3)

plt.tight_layout()
plt.savefig('spaceinfra_backtest_performance.png', dpi=150, bbox_inches='tight')
print("  ✓ Saved to spaceinfra_backtest_performance.png")

# Individual constituent performance
print("\nIndividual Constituent Performance:")
print("-" * 80)

constituent_performance = []
for col in df_prices.columns:
    stock_returns = returns[col]
    stock_metrics = calculate_metrics(stock_returns, col)
    weight = weights.get(col, 0)

    constituent_performance.append({
        'Ticker': col,
        'Weight': f"{weight*100:.1f}%",
        'Total Return': f"{stock_metrics['total_return']:.2f}%",
        'Ann. Return': f"{stock_metrics['annualized_return']:.2f}%",
        'Volatility': f"{stock_metrics['volatility']:.2f}%",
        'Sharpe': f"{stock_metrics['sharpe_ratio']:.2f}",
    })

df_perf = pd.DataFrame(constituent_performance)
print(df_perf.to_string(index=False))

# Save results
results_summary = {
    'index_metrics': spaceinfra_metrics,
    'benchmark_metrics': spy_metrics if spy_cumulative is not None else None,
    'constituent_performance': constituent_performance,
}

import json
with open('backtest_results.json', 'w') as f:
    # Convert numpy types to native Python types
    def convert(obj):
        if isinstance(obj, np.integer):
            return int(obj)
        elif isinstance(obj, np.floating):
            return float(obj)
        elif isinstance(obj, np.ndarray):
            return obj.tolist()
        return obj

    json.dump(results_summary, f, indent=2, default=convert)

print("\n✓ Backtest results saved to backtest_results.json")
print("\n" + "="*80)
print("Backtest Complete!")
print("="*80)
