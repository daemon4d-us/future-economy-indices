// Index engine crate - Weighting algorithm and index calculation

pub mod backtest;
pub mod weighting;

pub use weighting::{CompanyMetrics, IndexConstituent, IndexSummaryStats, WeightingAlgorithm};
