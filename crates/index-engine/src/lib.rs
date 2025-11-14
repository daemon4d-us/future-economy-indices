// Index engine crate - Weighting algorithm and index calculation

pub mod weighting;
pub mod backtest;

pub use weighting::{
    CompanyMetrics, IndexConstituent, IndexSummaryStats, WeightingAlgorithm,
};
