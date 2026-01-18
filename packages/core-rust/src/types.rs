//! Type definitions for trading fitness analysis.

use serde::{Deserialize, Serialize};

/// Result of excess gain/loss calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcessGainLossResult {
    /// Excess gains at each point.
    pub excess_gains: Vec<f64>,
    /// Excess losses at each point.
    pub excess_losses: Vec<f64>,
    /// Number of ITH epochs identified.
    pub num_of_ith_epochs: usize,
    /// Boolean flags for ITH epochs.
    pub ith_epochs: Vec<bool>,
    /// Coefficient of variation of ITH interval lengths.
    pub ith_intervals_cv: f64,
}

/// NAV record for analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavRecord {
    /// Date string (ISO 8601).
    pub date: String,
    /// Net Asset Value.
    pub nav: f64,
    /// Profit and Loss (optional).
    pub pnl: Option<f64>,
}

/// Fitness metrics for a trading strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitnessMetrics {
    /// Annualized Sharpe ratio.
    pub sharpe_ratio: f64,
    /// Maximum drawdown as decimal.
    pub max_drawdown: f64,
    /// Total return as decimal.
    pub total_return: f64,
    /// Number of trading days.
    pub trading_days: usize,
}

/// ITH analysis result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IthResult {
    /// Unique identifier.
    pub uid: String,
    /// TMAEG threshold used.
    pub tmaeg: f64,
    /// Number of ITH epochs.
    pub ith_epochs: usize,
    /// Coefficient of variation of ITH intervals.
    pub ith_intervals_cv: f64,
    /// Whether strategy qualifies.
    pub qualified: bool,
    /// Fitness metrics.
    pub metrics: FitnessMetrics,
}
