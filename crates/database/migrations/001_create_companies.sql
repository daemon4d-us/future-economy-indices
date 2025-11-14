-- Create companies table

CREATE TABLE companies (
    id SERIAL PRIMARY KEY,
    ticker VARCHAR(10) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    market_cap BIGINT,
    space_score REAL,
    ai_score REAL,
    segments TEXT[],
    last_classified_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

CREATE INDEX idx_companies_ticker ON companies(ticker);
CREATE INDEX idx_companies_space_score ON companies(space_score);
CREATE INDEX idx_companies_ai_score ON companies(ai_score);
