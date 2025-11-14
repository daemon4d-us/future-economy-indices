-- Create index_compositions table

CREATE TABLE index_compositions (
    id SERIAL PRIMARY KEY,
    index_name VARCHAR(50) NOT NULL,
    rebalance_date DATE NOT NULL,
    company_id INTEGER REFERENCES companies(id) ON DELETE CASCADE,
    weight REAL NOT NULL,
    rank INTEGER,
    space_revenue_pct REAL,
    revenue_growth_rate REAL,
    reason_included TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    UNIQUE(index_name, rebalance_date, company_id)
);

CREATE INDEX idx_index_compositions_name_date ON index_compositions(index_name, rebalance_date);
