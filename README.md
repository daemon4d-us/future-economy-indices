# Future Economy Indices

Two-index newsletter business: **SPACEINFRA** (Space Infrastructure) and **AIINFRA** (AI Infrastructure).

## Project Status

✅ **Research Phase Complete**
- SPACEINFRA index: 20 companies, 83% backtest return, 1.53 Sharpe ratio
- AI classification system operational
- 3-factor weighting algorithm validated

🚧 **Production Phase Started**
- Rust system architecture initialized
- Database schema defined
- API server skeleton created

## Quick Start

### Prerequisites
- Rust 1.70+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- PostgreSQL 14+
- Polygon.io API key
- Anthropic API key

### Setup

1. **Clone and setup environment**
```bash
cd future-economy-indices
cp .env.example .env
# Edit .env with your API keys and database URL
```

2. **Setup PostgreSQL database**
```bash
createdb future_economy_indices

# Run migrations
cd crates/database
sqlx database setup
```

3. **Build project**
```bash
cargo build
```

4. **Run API server**
```bash
cargo run --bin api-server
```

5. **Test health endpoint**
```bash
curl http://localhost:3000/health
```

## Project Structure

```
future-economy-indices/
├── crates/
│   ├── database/           # PostgreSQL models & migrations
│   ├── data-ingestion/     # Polygon.io API client
│   ├── ai-classifier/      # Anthropic AI classification
│   ├── index-engine/       # Weighting algorithm & backtesting
│   └── api-server/         # REST API (Axum)
├── research/               # Python prototypes & analysis
│   ├── polygon_client.py
│   ├── ai_classifier.py
│   ├── weighting_algorithm.py
│   └── *.ipynb            # Jupyter notebooks
├── PRODUCTION_ROADMAP.md  # Detailed production plan
└── Cargo.toml             # Workspace manifest
```

## Research Results (Python Prototype)

### SPACEINFRA Index (20 companies)

**Performance (1 Year)**
- Total Return: 83.37%
- Annualized: 84.71%
- Sharpe Ratio: 1.53
- Max Drawdown: -34.28%

**Top Holdings**
1. RKLB - Rocket Lab (10.0%)
2. ASTS - AST SpaceMobile (8.6%)
3. LUNR - Intuitive Machines (8.5%)

**Index Characteristics**
- Space Revenue %: 38.7% (weighted avg)
- Revenue Growth: 34.7%
- Total Market Cap: $643.7B

## Development Roadmap

See [PRODUCTION_ROADMAP.md](./PRODUCTION_ROADMAP.md) for detailed timeline.

### Phase 1: Rust Production System (Weeks 1-3)
- [x] Project structure initialized
- [x] Database schema designed
- [ ] Port Python prototype to Rust
- [ ] CLI tools for data management
- [ ] Automated testing

### Phase 2: Website (Weeks 2-4)
- [ ] Next.js frontend
- [ ] Public index pages
- [ ] Newsletter signup
- [ ] Stripe payment integration

### Phase 3: Newsletter (Weeks 3-5)
- [ ] ConvertKit/Beehiiv integration
- [ ] Content generation templates
- [ ] Automated quarterly reports
- [ ] Email delivery

### Phase 4: Launch (Weeks 5-6)
- [ ] Beta testing
- [ ] Public launch
- [ ] Marketing campaign

## Technology Stack

| Component | Technology |
|-----------|------------|
| Core Engine | Rust |
| Database | PostgreSQL |
| API Server | Axum |
| Frontend | Next.js + TailwindCSS |
| Newsletter | ConvertKit |
| Payments | Stripe |
| Hosting | Fly.io + Vercel |

## Revenue Targets

- **Year 1**: $150K ARR (500 paid subscribers @ $99/year)
- **Year 2**: $500K ARR
- **Year 3**: $1M ARR → Launch ETFs

## Commands

```bash
# Build all crates
cargo build

# Run tests
cargo test

# Run API server
cargo run --bin api-server

# Check code quality
cargo clippy
cargo fmt

# Database migrations
cd crates/database && sqlx migrate run
```

## Contributing

This is a private project for now. See CLAUDE.md for AI assistant guidance.

## License

Proprietary - All Rights Reserved

## CI/CD Status

[![Build and Push Docker Image](https://github.com/daemon4d-us/future-economy-indices/actions/workflows/build.yml/badge.svg)](https://github.com/daemon4d-us/future-economy-indices/actions/workflows/build.yml)
[![Deploy to GKE with Helm](https://github.com/daemon4d-us/future-economy-indices/actions/workflows/deploy.yml/badge.svg)](https://github.com/daemon4d-us/future-economy-indices/actions/workflows/deploy.yml)
