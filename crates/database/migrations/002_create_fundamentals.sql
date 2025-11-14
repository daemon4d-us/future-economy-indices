-- Create fundamentals table

CREATE TABLE fundamentals (
    id SERIAL PRIMARY KEY,
    company_id INTEGER REFERENCES companies(id) ON DELETE CASCADE,
    date DATE NOT NULL,
    revenue BIGINT,
    revenue_growth_yoy REAL,
    revenue_growth_3y_cagr REAL,
    market_cap BIGINT,
    price REAL,
    volume BIGINT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    UNIQUE(company_id, date)
);

CREATE INDEX idx_fundamentals_company_date ON fundamentals(company_id, date);
