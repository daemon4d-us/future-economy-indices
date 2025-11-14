# Future Economy Indices - Production Roadmap

**Goal**: Launch SPACEINFRA & AIINFRA newsletter business targeting $150K ARR Year 1

---

## Current Status: Research Phase Complete ✅

### Achieved:
- ✅ SPACEINFRA index with 20 companies
- ✅ AI classification system (Claude)
- ✅ 3-factor weighting algorithm (40% space revenue, 30% market cap, 30% growth)
- ✅ Backtest: 83% return, 1.53 Sharpe ratio
- ✅ Python prototype proving concept

### Ready For:
- 🚀 Production Rust system
- 🚀 Website and newsletter infrastructure
- 🚀 Launch preparation

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     FUTURE ECONOMY INDICES                  │
│                    Two-Index Newsletter Business             │
└─────────────────────────────────────────────────────────────┘

┌────────────────────┐        ┌────────────────────┐
│   SPACEINFRA       │        │    AIINFRA         │
│   Space Index      │        │    AI Index        │
│   (20 companies)   │        │    (TBD)           │
└────────────────────┘        └────────────────────┘
         │                             │
         └─────────────┬───────────────┘
                       │
         ┌─────────────▼──────────────┐
         │   RUST CORE ENGINE         │
         │   - Data ingestion         │
         │   - Classification         │
         │   - Index calculation      │
         │   - Quarterly rebalancing  │
         │   - Performance tracking   │
         └─────────────┬──────────────┘
                       │
         ┌─────────────▼──────────────┐
         │   POSTGRESQL DATABASE      │
         │   - Companies & financials │
         │   - Index composition      │
         │   - Historical data        │
         │   - Subscriber data        │
         └─────────────┬──────────────┘
                       │
         ┌─────────────▼──────────────┐
         │   WEBSITE + NEWSLETTER     │
         │   - Public index pages     │
         │   - Newsletter delivery    │
         │   - Stripe payment         │
         │   - User dashboard         │
         └────────────────────────────┘
```

---

## Phase 1: Rust Production System (Weeks 1-3)

### Goal: Replace Python prototype with production-grade Rust system

### 1.1 Project Setup
- Initialize Rust workspace with multiple crates
- Set up PostgreSQL database with migrations
- Configure environment management (.env)
- Set up CI/CD pipeline (GitHub Actions)

### 1.2 Core Modules

#### Module: `data-ingestion`
**Purpose**: Fetch market data from Polygon.io
- Stock tickers and metadata
- Financial fundamentals (revenue, growth)
- Historical prices (OHLCV)
- Market cap data
- Rate limiting and retry logic
- Data caching to minimize API calls

#### Module: `ai-classifier`
**Purpose**: AI-powered company classification
- Anthropic API integration
- Company classification prompts
- Space/AI revenue % estimation
- Segment categorization
- Confidence scoring
- Batch processing

#### Module: `index-engine`
**Purpose**: Index calculation and management
- 3-factor weighting algorithm
- Position size constraints
- Index composition generation
- Quarterly rebalancing logic
- Performance calculation
- Backtest simulation

#### Module: `database`
**Purpose**: PostgreSQL data layer
- Schema migrations (sqlx)
- Company models
- Index composition history
- Performance metrics
- Subscriber management

### 1.3 CLI Tools

```bash
# Data management
cargo run --bin ingest -- fetch-universe --exchange NASDAQ NYSE
cargo run --bin ingest -- update-financials --ticker ASTS
cargo run --bin ingest -- classify-company --ticker RKLB

# Index operations
cargo run --bin index -- calculate --index SPACEINFRA --date 2025-11-10
cargo run --bin index -- rebalance --index SPACEINFRA --quarter Q1-2025
cargo run --bin index -- backtest --index SPACEINFRA --from 2024-01-01

# Newsletter
cargo run --bin newsletter -- generate --index SPACEINFRA --quarter Q4-2024
cargo run --bin newsletter -- send --tier free
```

### 1.4 Database Schema

```sql
-- Companies table
CREATE TABLE companies (
    id SERIAL PRIMARY KEY,
    ticker VARCHAR(10) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    market_cap BIGINT,
    space_score FLOAT,
    ai_score FLOAT,
    segments TEXT[],
    last_classified_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Financials table
CREATE TABLE fundamentals (
    id SERIAL PRIMARY KEY,
    company_id INTEGER REFERENCES companies(id),
    date DATE NOT NULL,
    revenue BIGINT,
    revenue_growth_yoy FLOAT,
    revenue_growth_3y_cagr FLOAT,
    market_cap BIGINT,
    price FLOAT,
    volume BIGINT,
    created_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(company_id, date)
);

-- Index composition history
CREATE TABLE index_compositions (
    id SERIAL PRIMARY KEY,
    index_name VARCHAR(50) NOT NULL,  -- SPACEINFRA, AIINFRA
    rebalance_date DATE NOT NULL,
    company_id INTEGER REFERENCES companies(id),
    weight FLOAT NOT NULL,
    rank INTEGER,
    space_revenue_pct FLOAT,
    revenue_growth_rate FLOAT,
    reason_included TEXT,
    created_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(index_name, rebalance_date, company_id)
);

-- Index performance
CREATE TABLE index_performance (
    id SERIAL PRIMARY KEY,
    index_name VARCHAR(50) NOT NULL,
    date DATE NOT NULL,
    value FLOAT NOT NULL,
    daily_return FLOAT,
    created_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(index_name, date)
);

-- Subscribers
CREATE TABLE subscribers (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    tier VARCHAR(20) NOT NULL,  -- free, paid
    stripe_customer_id VARCHAR(255),
    stripe_subscription_id VARCHAR(255),
    subscribed_at TIMESTAMP DEFAULT NOW(),
    unsubscribed_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Newsletter sends
CREATE TABLE newsletter_sends (
    id SERIAL PRIMARY KEY,
    index_name VARCHAR(50) NOT NULL,
    quarter VARCHAR(10) NOT NULL,  -- Q1-2025
    tier VARCHAR(20) NOT NULL,
    sent_at TIMESTAMP DEFAULT NOW(),
    recipient_count INTEGER,
    open_rate FLOAT,
    click_rate FLOAT
);
```

---

## Phase 2: Website Design & Architecture (Weeks 2-4)

### Goal: Public-facing website for index data and newsletter signup

### 2.1 Tech Stack Options

#### Option A: Static Site + API (Recommended for MVP)
- **Frontend**: Next.js (React) + TailwindCSS
- **Backend**: Rust API (Axum or Actix-web)
- **Database**: PostgreSQL
- **Hosting**: Vercel (frontend) + Railway/Fly.io (backend)
- **Pros**: Fast, cheap, scalable, modern
- **Cons**: Two deployments to manage

#### Option B: Full Rust Stack
- **Framework**: Leptos or Yew (Rust WASM)
- **Backend**: Axum + PostgreSQL
- **Hosting**: Single deployment on Fly.io
- **Pros**: Single language, fast, unified deployment
- **Cons**: Steeper learning curve, fewer resources

**Recommendation**: Option A for MVP, Option B for v2

### 2.2 Website Pages

#### Public Pages:
1. **Homepage** (`/`)
   - Overview of both indices
   - Latest performance stats
   - Newsletter signup CTA
   - Value proposition

2. **SPACEINFRA Page** (`/spaceinfra`)
   - Current index composition (20 companies)
   - Performance chart (1Y, 3Y, 5Y, All-time)
   - Top holdings table
   - Segment breakdown
   - Methodology explanation
   - Historical rebalancing

3. **AIINFRA Page** (`/aiinfra`)
   - Same structure as SPACEINFRA

4. **Methodology** (`/methodology`)
   - AI classification process
   - 3-factor weighting algorithm
   - Rebalancing schedule (quarterly)
   - Data sources
   - Transparency and accountability

5. **Pricing** (`/pricing`)
   - Free tier: Monthly newsletter, basic data
   - Paid tier ($99/year): Weekly updates, full data, early access
   - Enterprise: Custom data feeds

6. **About** (`/about`)
   - Mission and vision
   - Team (if applicable)
   - Contact

#### User Dashboard (Paid Tier):
- Index holdings with real-time prices
- Performance analytics
- Downloadable data (CSV, JSON)
- Historical rebalancing data
- API access (future)

### 2.3 Design Principles

**Brand Identity:**
- **Colors**: Space-themed (dark blues, purples, teals)
- **Typography**: Modern, tech-forward (Inter, Space Grotesk)
- **Style**: Data-driven, professional, futuristic

**Key Features:**
- 📊 Interactive performance charts (Chart.js or Recharts)
- 📈 Real-time index data
- 🎨 Clean, minimal design
- 📱 Mobile-responsive
- ⚡ Fast load times (<2s)

### 2.4 API Endpoints

```
GET  /api/indices                    # List all indices
GET  /api/indices/:name               # Get index details
GET  /api/indices/:name/composition   # Current holdings
GET  /api/indices/:name/performance   # Historical performance
GET  /api/indices/:name/history       # Rebalancing history
POST /api/subscribe                   # Newsletter signup
POST /api/webhooks/stripe             # Stripe payment webhook
```

---

## Phase 3: Newsletter Infrastructure (Weeks 3-5)

### Goal: Automated newsletter generation and delivery

### 3.1 Newsletter Tech Stack

**Option A: Managed Service (Recommended for MVP)**
- **Platform**: ConvertKit, Beehiiv, or Substack
- **Integration**: Stripe for payments
- **Pros**: Fast setup, built-in features, compliance
- **Cons**: Monthly cost, less customization

**Option B: Self-hosted**
- **Email**: SendGrid or Postmark API
- **Generation**: Rust templates (Tera or Handlebars)
- **Storage**: PostgreSQL for subscribers
- **Pros**: Full control, lower cost at scale
- **Cons**: More complex, compliance burden (GDPR, CAN-SPAM)

**Recommendation**: Option A for MVP, Option B when >1000 subscribers

### 3.2 Newsletter Content Structure

#### Free Tier (Monthly):
**Subject**: "SPACEINFRA Q4 2024: Index Update & Performance"

```
┌─────────────────────────────────────────┐
│  SPACEINFRA QUARTERLY REPORT            │
│  Q4 2024                                │
└─────────────────────────────────────────┘

📊 PERFORMANCE SUMMARY
- Quarterly Return: +XX%
- YTD Return: +XX%
- vs S&P 500: +XX%
- Sharpe Ratio: X.XX

🔝 TOP HOLDINGS (As of Dec 31, 2024)
1. RKLB - Rocket Lab (10.0%)
2. ASTS - AST SpaceMobile (8.6%)
3. LUNR - Intuitive Machines (8.5%)
...

📈 REBALANCING CHANGES
- Added: [Company X, Company Y]
- Removed: [Company Z]
- Weight Changes: [Notable adjustments]

🚀 SPACE INDUSTRY HIGHLIGHTS
- [Key news item 1]
- [Key news item 2]
- [Key news item 3]

📖 METHODOLOGY REMINDER
- 20 companies, AI-selected
- 3-factor weighting (space %, market cap, growth)
- Quarterly rebalancing

[CTA: Upgrade to Paid for Weekly Updates]
```

#### Paid Tier ($99/year):
**All of above PLUS:**
- Weekly market commentary
- Individual stock deep dives
- Early access to rebalancing (1 week before)
- Downloadable data (CSV/Excel)
- Priority email support

### 3.3 Content Generation Pipeline

```
┌─────────────────────────────────────────┐
│  Quarterly (Automated)                  │
│  1. Run index rebalancing               │
│  2. Calculate performance metrics       │
│  3. Generate newsletter content (Rust)  │
│  4. Human review & edits                │
│  5. Schedule send via ConvertKit API    │
│  6. Track open/click rates             │
└─────────────────────────────────────────┘
```

### 3.4 Monetization Strategy

**Pricing Tiers:**
- **Free**:
  - Monthly newsletter (2 indices)
  - Basic performance data
  - Public website access
  - Target: Build audience (1000+ subscribers)

- **Paid ($99/year or $10/month)**:
  - Weekly updates
  - Full historical data
  - Downloadable reports
  - Early rebalancing access
  - Target: 1,500 paid = $150K ARR

- **Enterprise (Custom pricing)**:
  - API access
  - Custom indices
  - Bulk data exports
  - White-label options
  - Target: 5-10 clients at $5K-$10K each

**Year 1 Revenue Projections:**
```
Month 1-3:   100 free, 10 paid = $1K/month
Month 4-6:   500 free, 50 paid = $5K/month
Month 7-9:  1000 free, 200 paid = $20K/month
Month 10-12: 2000 free, 500 paid = $50K/month

Year 1 Total: ~$150K ARR ✅
```

---

## Phase 4: Launch Preparation (Weeks 5-6)

### 4.1 Pre-Launch Checklist

**Technical:**
- ✅ Rust system deployed and tested
- ✅ Website live with both indices
- ✅ Newsletter infrastructure connected
- ✅ Payment processing (Stripe) working
- ✅ Database backups automated
- ✅ Monitoring and alerts set up

**Content:**
- ✅ Initial 2 quarters of historical data
- ✅ Methodology page written
- ✅ About page and brand story
- ✅ Sample newsletters created
- ✅ FAQs written

**Legal/Compliance:**
- ✅ Terms of Service
- ✅ Privacy Policy (GDPR compliant)
- ✅ Disclaimer (not investment advice)
- ✅ CAN-SPAM compliance for emails

**Marketing:**
- ✅ Landing page with waitlist
- ✅ Social media accounts (Twitter/X, LinkedIn)
- ✅ Press kit and one-pager
- ✅ Initial content marketing plan

### 4.2 Launch Strategy

**Soft Launch (Week 6):**
- Release to friends, family, beta testers
- Free tier only
- Gather feedback
- Fix bugs

**Public Launch (Week 8):**
- Announce on social media
- Post on HN, Reddit (r/investing, r/space)
- Email personal network
- Publish methodology blog post
- Enable paid tier

**Growth Tactics:**
- SEO-optimized content (space investing, AI stocks)
- Guest posts on finance blogs
- Twitter/X engagement with space/AI communities
- Partner with space industry newsletters
- Podcast appearances

---

## Timeline Summary

| Week | Focus | Deliverables |
|------|-------|--------------|
| 1-2 | Rust Core | Database, data ingestion, basic CLI |
| 3 | Rust Core | AI classifier, index engine, tests |
| 2-4 | Website | Design, frontend, API endpoints |
| 3-5 | Newsletter | ConvertKit setup, templates, automation |
| 5 | Integration | Connect all systems, end-to-end testing |
| 6 | Content | Write newsletters, methodology, about page |
| 7 | Beta | Soft launch, gather feedback |
| 8 | Launch | Public announcement, enable payments |
| 9-12 | Growth | Marketing, content, iterate |

---

## Technology Choices Summary

| Component | Technology | Why |
|-----------|------------|-----|
| Core Engine | **Rust** | Performance, safety, modern tooling |
| Database | **PostgreSQL** | Reliable, SQL, mature ecosystem |
| Website Frontend | **Next.js + TailwindCSS** | Fast development, great DX, SEO-friendly |
| Website Backend | **Rust (Axum)** | Type-safe API, shares code with core |
| Hosting (Frontend) | **Vercel** | Zero-config, fast, CDN |
| Hosting (Backend) | **Fly.io or Railway** | Easy Rust deployments, good pricing |
| Newsletter | **ConvertKit or Beehiiv** | Built-in features, compliance, analytics |
| Payments | **Stripe** | Industry standard, great docs |
| Email Delivery | **SendGrid or Postmark** | Reliable, good deliverability |
| Monitoring | **Sentry + Datadog** | Error tracking, performance monitoring |

---

## Risk Mitigation

### Technical Risks:
- **Data provider outages**: Cache aggressively, have backup data sources
- **AI API failures**: Implement retries, fallbacks, manual override
- **Database failures**: Daily backups, hot standby

### Business Risks:
- **Low subscriber growth**: Strong SEO, content marketing, partnerships
- **Churn**: High-quality content, consistent delivery, engagement
- **Competition**: Unique AI-driven methodology, transparency, niche focus

### Legal Risks:
- **Investment advice claims**: Clear disclaimers, educational content
- **Data accuracy**: Regular audits, transparent methodology
- **Compliance**: Legal review of T&C, privacy policy

---

## Success Metrics

### Technical:
- System uptime: >99%
- API response time: <500ms p95
- Newsletter delivery rate: >98%
- Database query time: <100ms p95

### Business:
- **Month 1**: 100 free subscribers
- **Month 3**: 500 free, 25 paid ($2.5K MRR)
- **Month 6**: 1000 free, 100 paid ($10K MRR)
- **Month 12**: 2000 free, 500 paid ($50K MRR) → $150K ARR

### Content:
- Email open rate: >30%
- Click-through rate: >5%
- Unsubscribe rate: <2%
- Newsletter NPS: >40

---

## Next Immediate Steps

1. **Create Rust workspace structure**
2. **Set up PostgreSQL database**
3. **Implement data ingestion module**
4. **Port Python prototype logic to Rust**
5. **Design website mockups**
6. **Set up ConvertKit account**

Ready to begin! 🚀
