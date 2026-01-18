//! Trading metrics calculations.

use crate::types::FitnessMetrics;

/// Calculate the Sharpe ratio of returns.
///
/// # Arguments
/// * `returns` - Array of periodic returns
/// * `periods_per_year` - Number of periods per year (252 for daily stocks, 365 for crypto)
/// * `risk_free_rate` - Risk-free rate (default 0.0)
///
/// # Returns
/// Annualized Sharpe ratio, or NaN if calculation is not possible.
pub fn sharpe_ratio(returns: &[f64], periods_per_year: f64, risk_free_rate: f64) -> f64 {
    if returns.len() < 2 {
        return f64::NAN;
    }

    // Filter out NaN values
    let valid_returns: Vec<f64> = returns.iter().filter(|r| !r.is_nan()).copied().collect();

    if valid_returns.len() < 2 {
        return f64::NAN;
    }

    let n = valid_returns.len() as f64;
    let mean: f64 = valid_returns.iter().sum::<f64>() / n;

    // Sample standard deviation
    let variance: f64 =
        valid_returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let std_dev = variance.sqrt();

    if std_dev == 0.0 {
        return f64::NAN;
    }

    let excess_return = mean - risk_free_rate;
    periods_per_year.sqrt() * (excess_return / std_dev)
}

/// Calculate the maximum drawdown of a NAV series.
///
/// # Arguments
/// * `nav_values` - Array of NAV values
///
/// # Returns
/// Maximum drawdown as a decimal (0.15 = 15% drawdown).
pub fn max_drawdown(nav_values: &[f64]) -> f64 {
    if nav_values.is_empty() {
        return 0.0;
    }

    let mut running_max = nav_values[0];
    let mut max_dd = 0.0;

    for &nav in nav_values.iter() {
        if nav > running_max {
            running_max = nav;
        }
        let drawdown = 1.0 - nav / running_max;
        if drawdown > max_dd {
            max_dd = drawdown;
        }
    }

    max_dd
}

/// Calculate total return from NAV series.
pub fn total_return(nav_values: &[f64]) -> f64 {
    if nav_values.len() < 2 {
        return 0.0;
    }
    let first = nav_values[0];
    let last = nav_values[nav_values.len() - 1];
    if first == 0.0 {
        return 0.0;
    }
    (last - first) / first
}

/// Calculate PnL (returns) from NAV series.
pub fn pnl_from_nav(nav_values: &[f64]) -> Vec<f64> {
    if nav_values.len() < 2 {
        return vec![0.0];
    }

    let mut pnl = vec![0.0];
    for i in 1..nav_values.len() {
        if nav_values[i - 1] != 0.0 {
            pnl.push((nav_values[i] - nav_values[i - 1]) / nav_values[i - 1]);
        } else {
            pnl.push(0.0);
        }
    }
    pnl
}

/// Calculate complete fitness metrics for a NAV series.
pub fn calculate_fitness_metrics(nav_values: &[f64], periods_per_year: f64) -> FitnessMetrics {
    let pnl = pnl_from_nav(nav_values);
    let sr = sharpe_ratio(&pnl, periods_per_year, 0.0);
    let mdd = max_drawdown(nav_values);
    let total_ret = total_return(nav_values);

    FitnessMetrics {
        sharpe_ratio: sr,
        max_drawdown: mdd,
        total_return: total_ret,
        trading_days: nav_values.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sharpe_ratio_positive() {
        let returns = vec![0.01, 0.02, 0.01, 0.015, 0.02];
        let sr = sharpe_ratio(&returns, 252.0, 0.0);
        assert!(sr > 0.0);
    }

    #[test]
    fn test_sharpe_ratio_negative() {
        let returns = vec![-0.01, -0.02, -0.01, -0.015, -0.02];
        let sr = sharpe_ratio(&returns, 252.0, 0.0);
        assert!(sr < 0.0);
    }

    #[test]
    fn test_sharpe_ratio_insufficient_data() {
        let returns = vec![0.01];
        let sr = sharpe_ratio(&returns, 252.0, 0.0);
        assert!(sr.is_nan());
    }

    #[test]
    fn test_max_drawdown_uptrend() {
        let nav = vec![1.0, 1.1, 1.2, 1.3, 1.4];
        let mdd = max_drawdown(&nav);
        assert_eq!(mdd, 0.0);
    }

    #[test]
    fn test_max_drawdown_downtrend() {
        let nav = vec![1.0, 0.9, 0.8, 0.7];
        let mdd = max_drawdown(&nav);
        assert!((mdd - 0.3).abs() < 0.001);
    }

    #[test]
    fn test_total_return() {
        let nav = vec![1.0, 1.1, 1.2];
        let ret = total_return(&nav);
        assert!((ret - 0.2).abs() < 0.001);
    }

    #[test]
    fn test_pnl_from_nav() {
        let nav = vec![1.0, 1.1, 1.21];
        let pnl = pnl_from_nav(&nav);
        assert_eq!(pnl.len(), 3);
        assert_eq!(pnl[0], 0.0);
        assert!((pnl[1] - 0.1).abs() < 0.001);
        assert!((pnl[2] - 0.1).abs() < 0.001);
    }
}
