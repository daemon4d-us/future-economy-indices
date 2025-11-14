# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Future Economy Indices is a two-index newsletter business:
1. **AIINFRA** - AI Infrastructure Index
2. **SPACEINFRA** - Space Infrastructure Index

Business model: Newsletter with paid tier, targeting $150K ARR Year 1, $500K Year 2, $1M Year 3 (then launch ETFs).

## Planned Technology Stack

- **Language**: Rust (shared codebase)
- **Database**: PostgreSQL
- **Market Data**: Polygon.io API
- **Rebalancing**: Quarterly

## Development Phases

1. CEF arbitrage system (completed)
2. AI Index screening (in progress)
3. Space Index screening
4. Website and newsletter infrastructure

## Architecture Principles

This project uses a shared Rust codebase approach, meaning both indices should share:
- Common market data fetching infrastructure
- Database schema and access patterns
- Screening and rebalancing logic (parameterized by index type)
- Newsletter generation utilities

Keep the core generic and configurable rather than duplicating code for each index.

## Expected Commands

Once the Rust project is initialized, standard commands will likely include:
- `cargo build` - Build the project
- `cargo test` - Run tests
- `cargo run` - Run the main application
- Database migrations will use a tool like `sqlx` or `diesel`

## Database Considerations

PostgreSQL schema should support:
- Stock universe tracking
- Index composition history (for quarterly rebalancing)
- Market data caching from Polygon.io
- Subscriber management (for newsletter tiers)
- Performance metrics for both indices

## Integration Points

- **Polygon.io**: Handle API rate limits, cache market data appropriately
- **Rebalancing**: Implement deterministic quarterly rebalancing logic with audit trail
- **Newsletter**: Generate consistent, data-driven reports from index performance
