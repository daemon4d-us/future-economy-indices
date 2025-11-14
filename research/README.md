# SPACEINFRA Research Prototype

Python-based research environment for developing the Space Infrastructure Index screening and weighting algorithm.

## Quick Start

### 1. Set up Python virtual environment

```bash
cd research
python3 -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate
```

### 2. Install dependencies

```bash
pip install -r requirements.txt
```

### 3. Configure API keys

Copy `.env.example` to `.env` and add your API keys:

```bash
cp .env.example .env
# Edit .env and add your POLYGON_API_KEY
```

### 4. Test the connection

```bash
python polygon_client.py
```

You should see output confirming connection to Polygon.io and data for ASTS (AST SpaceMobile).

### 5. Start exploring with Jupyter

```bash
jupyter notebook 01_explore_polygon_data.ipynb
```

## Project Structure

```
research/
├── polygon_client.py              # Polygon.io API client
├── 01_explore_polygon_data.ipynb  # Initial data exploration
├── requirements.txt               # Python dependencies
├── .env.example                   # Environment variables template
└── .env                          # Your API keys (git-ignored)
```

## Next Notebooks (TODO)

- `02_ai_classification.ipynb` - Build AI-powered company classifier
- `03_screening_criteria.ipynb` - Define fundamental screening rules
- `04_weighting_algorithm.ipynb` - Implement custom weighting (space rev %, market cap, growth)
- `05_backtest.ipynb` - Backtest index performance

## Workflow

1. **Data Collection** - Fetch company data from Polygon.io
2. **AI Classification** - Use LLM to identify space companies and estimate space revenue %
3. **Fundamental Screening** - Filter by market cap, liquidity, quality metrics
4. **Weighting** - Apply 3-factor algorithm (40% space rev, 30% market cap, 30% growth)
5. **Backtesting** - Validate strategy with historical data
6. **Production** - Migrate to Rust for production system
