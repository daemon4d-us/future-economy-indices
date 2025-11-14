# SPACEINFRA Index - Project Plan

## Overview
Build an AI-powered index screening system for Space Infrastructure companies using a hybrid approach: AI classification for company identification + fundamental screening for quality.

## Index Scope
Include all space infrastructure segments:
- Launch providers (SpaceX, Rocket Lab, etc.)
- Satellite manufacturers & operators
- Ground infrastructure (tracking, data centers, antennas)
- Space technology & components (propulsion, sensors, materials)

## Phase 1: Research Prototype (Python/Jupyter)

### 1.1 Environment Setup
- Create Python virtual environment
- Install dependencies: pandas, requests, openai/anthropic, jupyter
- Set up .env for API keys (Polygon.io, OpenAI/Anthropic)

### 1.2 Data Collection
- **Polygon.io Integration**
  - Fetch stock tickers and company metadata
  - Get financial fundamentals (market cap, revenue, etc.)
  - Download historical price data
  - Understand rate limits and caching strategy

- **Company Classification Data**
  - SEC filings (10-K business descriptions)
  - Company websites and investor relations
  - News articles and press releases
  - Industry classifications (SIC codes, keywords)

### 1.3 AI Classification System
- **Prompt Engineering**
  - Design prompts to classify companies as space-related
  - Extract space revenue percentage estimates
  - Identify specific space segments (launch, satellites, etc.)

- **Classification Pipeline**
  ```
  Company → Gather text data → AI analysis → Space score + segment tags
  ```

- **Validation**
  - Test on known space companies (SpaceX analogs, public satellite operators)
  - Test on edge cases (defense contractors with space divisions)
  - Build ground truth dataset for accuracy measurement

### 1.4 Fundamental Screening
Define screening criteria:
- **Minimum thresholds**
  - Market cap > $X million
  - Average daily volume > $X
  - Space revenue > Y% of total revenue (from AI estimate)

- **Quality filters**
  - Exclude penny stocks
  - Require minimum liquidity
  - Consider profitability/growth metrics

### 1.5 Custom Weighting Algorithm

**Primary Weighting Factors:**
1. **Space Revenue %** - Percentage of total revenue from space activities
2. **Market Cap** - Company size (with dampening to prevent over-concentration)
3. **Growth** - Revenue growth rate (YoY or multi-year CAGR)

**Proposed Algorithm:**
```python
# For each company in the index:
raw_score = (
    space_revenue_pct * w1 +           # 0-100%, weight ~0.4
    normalized_market_cap * w2 +        # Sqrt or log transform, weight ~0.3
    revenue_growth_rate * w3            # YoY or 3Y CAGR, weight ~0.3
)

# Normalize scores to sum to 1.0
weight = raw_score / sum(all_raw_scores)

# Apply constraints
weight = min(weight, max_position_size)   # e.g., 15%
weight = max(weight, min_position_size)   # e.g., 1%
```

**Normalization Details:**
- Space Revenue %: Already 0-100, can use directly
- Market Cap: Use `log(market_cap)` or `sqrt(market_cap)` to dampen large-cap dominance
- Growth: Clip extreme values (e.g., -50% to +200%), then normalize to 0-100 scale

**Constraints:**
- Maximum single position: 15% (prevent over-concentration)
- Minimum position: 1% (ensure meaningful allocation)
- Total constituents: 25-40 companies (diversification vs focus)

**Edge Cases to Handle:**
- Negative revenue growth: Use floor (e.g., min growth score = 0)
- Extreme growth (small base): Consider revenue scale in growth calculation
- Pure-play space (100% space revenue) vs conglomerates: Pure plays get bonus

### 1.6 Backtesting & Analysis
- Calculate historical index performance
- Compare to benchmarks (S&P 500, ARK Space ETF)
- Analyze quarterly rebalancing impacts
- Stress test with different parameters

## Phase 2: Production Implementation (Rust)

### 2.1 Database Schema (PostgreSQL)
```sql
-- Core tables needed:
companies (
  ticker, name, sector,
  space_score, space_segments,
  last_classified_at
)

fundamentals (
  ticker, date, market_cap,
  revenue, revenue_growth_yoy,
  revenue_growth_3y_cagr,
  space_revenue_pct,  -- from AI classification
  volume, price
)

index_composition (
  rebalance_date, ticker, weight,
  reason_included, metrics_snapshot
)

market_data_cache (
  ticker, date, ohlcv,
  fetched_at
)
```

### 2.2 Rust Architecture
```
src/
  main.rs                    # CLI entry point
  lib.rs                     # Library root

  data/
    polygon.rs               # Polygon.io API client
    ai_classifier.rs         # AI API integration
    cache.rs                 # Market data caching

  screening/
    discovery.rs             # Find candidate companies
    classifier.rs            # AI classification pipeline
    fundamentals.rs          # Fundamental screening
    scorer.rs                # Custom weighting algorithm

  index/
    composition.rs           # Index composition logic
    rebalancing.rs           # Quarterly rebalancing
    performance.rs           # Performance tracking

  db/
    models.rs                # Database models
    migrations/              # SQL migrations
```

### 2.3 Key Rust Crates
- `sqlx` - Async PostgreSQL with compile-time query checking
- `reqwest` - HTTP client for APIs
- `tokio` - Async runtime
- `serde` - Serialization
- `chrono` - Date/time handling
- `anyhow` - Error handling
- `clap` - CLI argument parsing

### 2.4 CLI Commands
```bash
# Data collection
cargo run -- fetch-universe --market nasdaq,nyse
cargo run -- update-fundamentals --ticker ASTS
cargo run -- classify-company --ticker ASTS

# Screening
cargo run -- screen-candidates --min-marketcap 100000000
cargo run -- generate-index --date 2024-12-31

# Rebalancing
cargo run -- rebalance --quarter Q1-2025 --dry-run
cargo run -- backtest --start 2020-01-01 --end 2024-12-31

# Reporting
cargo run -- index-performance --quarter Q4-2024
cargo run -- export-composition --format csv
```

## Phase 3: Automation & Newsletter

### 3.1 Scheduled Jobs
- Daily: Update market data for index constituents
- Weekly: Re-classify new IPOs and SPACs
- Quarterly: Generate rebalancing report
- On-demand: Newsletter generation with performance stats

### 3.2 Newsletter Content
- Index performance vs benchmarks
- Constituent changes (adds/removes)
- Top performers and laggards
- Space industry news highlights
- Sector allocation breakdown

## Key Decision Points

### AI Model Selection
- **Option A: OpenAI GPT-4** - Best accuracy, moderate cost
- **Option B: Anthropic Claude** - Good accuracy, faster, better for analysis
- **Option C: Open source (Llama)** - Lower cost, requires more tuning

### Data Sources
- **Primary: Polygon.io** - Comprehensive, good API
- **Alternative: Alpha Vantage, Yahoo Finance** - Free options for prototyping
- **Fundamental data: SEC EDGAR API** - Free company filings

### Rebalancing Logic
- Fixed quarterly dates vs floating (3-month intervals)
- Constituent change thresholds (when to add/remove)
- Turnover constraints (limit churn)

## Success Metrics

### Research Phase
- AI classifier accuracy > 90% on test set
- Identify 30+ high-quality space infrastructure stocks
- Backtest shows reasonable Sharpe ratio (> 0.5)

### Production Phase
- End-to-end rebalancing runs in < 5 minutes
- Data freshness: market data < 1 day old
- System uptime > 99%

## Next Steps

1. **Immediate:** Set up Python research environment
2. **Week 1:** Build Polygon.io integration and explore data
3. **Week 2:** Develop AI classification prompts and test
4. **Week 3:** Define screening criteria and weighting algorithm
5. **Week 4:** Backtest and validate approach
6. **Week 5+:** Begin Rust implementation if research validates approach

## Questions to Resolve

### Weighting Algorithm (answers will refine implementation)
1. **Factor weights**: Should space revenue % be weighted more heavily than the other two factors? (Suggested: 40% space revenue, 30% market cap, 30% growth)
2. **Growth metric**: Use YoY growth, 3-year CAGR, or both?
3. **Market cap transformation**: Use log or sqrt to dampen large-cap dominance?

### Index Construction
4. What should the target number of constituents be (10? 30? 50?)? (Suggested: 25-40)
5. Should we start with US-only or include international stocks?
6. What benchmark should we compare against? (ARK Space ETF ARKX, S&P 500, custom blend?)

### Technical Setup
7. Do you have Polygon.io and AI API accounts set up?
8. Which AI provider: OpenAI (GPT-4) or Anthropic (Claude)? Both have pros/cons.
