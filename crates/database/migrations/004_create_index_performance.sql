-- Create index_performance table

CREATE TABLE index_performance (
    id SERIAL PRIMARY KEY,
    index_name VARCHAR(50) NOT NULL,
    date DATE NOT NULL,
    value REAL NOT NULL,
    daily_return REAL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    UNIQUE(index_name, date)
);

CREATE INDEX idx_index_performance_name_date ON index_performance(index_name, date);
