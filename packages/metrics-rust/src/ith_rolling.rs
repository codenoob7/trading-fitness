//! Rolling window ITH computation for columnar feature generation.
//!
//! Computes time-agnostic ITH features over sliding windows, producing
//! bounded [0, 1] outputs suitable for LSTM/BiLSTM consumption.
//!
//! # Per-Window TMAEG Calculation (Simplest Form)
//!
//! The TMAEG (Target Maximum Acceptable Excess Gain) threshold uses the
//! **mathematically simplest** definition:
//!
//! - **Bull ITH**: TMAEG = Maximum Drawdown of the window
//! - **Bear ITH**: TMAEG = Maximum Runup of the window
//!
//! This approach is elegant because:
//! 1. TMAEG is derived directly from the window's own extremes
//! 2. No arbitrary parameters or percentile tuning
//! 3. Epochs trigger when gains exceed the maximum adverse movement
//! 4. Mathematically symmetric: drawdown ↔ runup

use crate::ith::{bear_ith, bull_ith};
use crate::ith_normalize::{
    normalize_cv, normalize_drawdown, normalize_epochs, normalize_excess, normalize_runup,
};

/// Rolling ITH features - all bounded [0, 1] for LSTM consumption.
///
/// Each field is a vector of length N (same as input NAV), where
/// the first `lookback - 1` values are NaN (insufficient data).
#[derive(Debug, Clone)]
pub struct RollingIthFeatures {
    /// Bull epoch density: epochs / expected_epochs, saturated to [0, 1]
    pub bull_epoch_density: Vec<f64>,
    /// Bear epoch density: epochs / expected_epochs, saturated to [0, 1]
    pub bear_epoch_density: Vec<f64>,
    /// Bull excess gain (sum in window): tanh-normalized to [0, 1]
    pub bull_excess_gain: Vec<f64>,
    /// Bear excess gain (sum in window): tanh-normalized to [0, 1]
    pub bear_excess_gain: Vec<f64>,
    /// Bull intervals CV: sigmoid-normalized to [0, 1]
    pub bull_cv: Vec<f64>,
    /// Bear intervals CV: sigmoid-normalized to [0, 1]
    pub bear_cv: Vec<f64>,
    /// Max drawdown in window: already [0, 1]
    pub max_drawdown: Vec<f64>,
    /// Max runup in window: already [0, 1]
    pub max_runup: Vec<f64>,
}

impl RollingIthFeatures {
    /// Create a new RollingIthFeatures with pre-allocated vectors.
    fn new(len: usize) -> Self {
        Self {
            bull_epoch_density: vec![f64::NAN; len],
            bear_epoch_density: vec![f64::NAN; len],
            bull_excess_gain: vec![f64::NAN; len],
            bear_excess_gain: vec![f64::NAN; len],
            bull_cv: vec![f64::NAN; len],
            bear_cv: vec![f64::NAN; len],
            max_drawdown: vec![f64::NAN; len],
            max_runup: vec![f64::NAN; len],
        }
    }
}

/// Compute Maximum Drawdown for Bull ITH TMAEG.
///
/// Maximum Drawdown = 1 - (trough / peak)
///
/// This is the simplest, most mathematically pure definition:
/// - An epoch triggers when excess gain exceeds the maximum adverse movement
/// - No arbitrary parameters, no percentile tuning
/// - TMAEG is derived directly from the window's own extremes
///
/// # Arguments
/// * `window` - Normalized NAV window (first value = 1.0)
///
/// # Returns
/// Maximum drawdown as a fraction [0, 1)
fn compute_max_drawdown(window: &[f64]) -> f64 {
    if window.len() < 2 {
        return f64::EPSILON;
    }

    let mut running_max = window[0];
    let mut max_drawdown = 0.0;

    for &val in window.iter().skip(1) {
        if val > running_max {
            running_max = val;
        }
        if running_max > 0.0 && val.is_finite() {
            let drawdown = 1.0 - val / running_max;
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }
    }

    // Ensure a minimum threshold to avoid division issues
    max_drawdown.max(f64::EPSILON)
}

/// Compute Maximum Runup for Bear ITH TMAEG.
///
/// Maximum Runup = 1 - (trough / peak) where we track the inverse:
/// - Running minimum (trough)
/// - Then measure how much price rises from that trough
///
/// This is the symmetric counterpart to Maximum Drawdown:
/// - Drawdown: how much price falls from peak (adverse for longs)
/// - Runup: how much price rises from trough (adverse for shorts)
///
/// # Arguments
/// * `window` - Normalized NAV window (first value = 1.0)
///
/// # Returns
/// Maximum runup as a fraction [0, 1)
fn compute_max_runup(window: &[f64]) -> f64 {
    if window.len() < 2 {
        return f64::EPSILON;
    }

    let mut running_min = window[0];
    let mut max_runup = 0.0;

    for &val in window.iter().skip(1) {
        if val < running_min {
            running_min = val;
        }
        if val > 0.0 && running_min > 0.0 && val.is_finite() {
            // Runup = how much price has risen from the trough
            // Formula: 1 - (trough / current) = (current - trough) / current
            let runup = 1.0 - running_min / val;
            if runup > max_runup {
                max_runup = runup;
            }
        }
    }

    // Ensure a minimum threshold to avoid division issues
    max_runup.max(f64::EPSILON)
}

/// Compute rolling ITH features over lookback windows.
///
/// This function computes Bull and Bear ITH metrics over sliding windows
/// of the NAV series, normalizing all outputs to [0, 1] for LSTM consumption.
///
/// **Key Design**: TMAEG uses the simplest mathematical definition:
/// - **Bull ITH**: TMAEG = Maximum Drawdown (the worst decline from peak)
/// - **Bear ITH**: TMAEG = Maximum Runup (the worst rise from trough)
///
/// This is elegant because epochs trigger when gains exceed the maximum
/// adverse movement in the window - no arbitrary parameters needed.
///
/// # Arguments
/// * `nav` - NAV series (N samples, typically starting near 1.0)
/// * `lookback` - Number of bars to look back for each computation
///
/// # Returns
/// `RollingIthFeatures` with shape (N,), where first `lookback-1` values are NaN
///
/// # Panics
/// Panics if `lookback` is 0 or greater than `nav.len()`
///
/// # Example
/// ```
/// use trading_fitness_metrics::ith_rolling::compute_rolling_ith;
///
/// let nav = vec![1.0, 1.01, 1.02, 1.015, 1.03, 1.025, 1.04, 1.05, 1.045, 1.06];
/// let features = compute_rolling_ith(&nav, 5);
///
/// // First 4 values (lookback-1) are NaN
/// assert!(features.bull_epoch_density[0].is_nan());
/// assert!(features.bull_epoch_density[3].is_nan());
///
/// // From index 4 onwards, values are bounded [0, 1]
/// let v = features.bull_epoch_density[4];
/// assert!(!v.is_nan() && v >= 0.0 && v <= 1.0);
/// ```
pub fn compute_rolling_ith(nav: &[f64], lookback: usize) -> RollingIthFeatures {
    assert!(lookback > 0, "lookback must be positive");
    assert!(
        lookback <= nav.len(),
        "lookback cannot exceed NAV length"
    );

    let n = nav.len();
    let mut features = RollingIthFeatures::new(n);

    // Compute features for each valid window position
    for i in (lookback - 1)..n {
        let window_start = i + 1 - lookback;
        let window = &nav[window_start..=i];

        // Renormalize window to start at 1.0 for consistent ITH computation
        let first_val = window[0];
        if first_val <= 0.0 || !first_val.is_finite() {
            // Skip invalid windows
            continue;
        }

        let normalized_window: Vec<f64> = window.iter().map(|v| v / first_val).collect();

        // SIMPLEST FORM: Use Maximum Drawdown/Runup as TMAEG
        // - Bull ITH: TMAEG = Max Drawdown (adverse for longs)
        // - Bear ITH: TMAEG = Max Runup (adverse for shorts)
        let bull_tmaeg = compute_max_drawdown(&normalized_window);
        let bear_tmaeg = compute_max_runup(&normalized_window);

        // Compute Bull ITH with max drawdown as TMAEG
        let bull_result = bull_ith(&normalized_window, bull_tmaeg);

        // Compute Bear ITH with max runup as TMAEG
        let bear_result = bear_ith(&normalized_window, bear_tmaeg);

        // Normalize and store features
        features.bull_epoch_density[i] = normalize_epochs(bull_result.num_of_epochs, lookback);
        features.bear_epoch_density[i] = normalize_epochs(bear_result.num_of_epochs, lookback);

        // Sum excess gains for the window
        let bull_excess_sum: f64 = bull_result.excess_gains.iter().sum();
        let bear_excess_sum: f64 = bear_result.excess_gains.iter().sum();
        features.bull_excess_gain[i] = normalize_excess(bull_excess_sum);
        features.bear_excess_gain[i] = normalize_excess(bear_excess_sum);

        features.bull_cv[i] = normalize_cv(bull_result.intervals_cv);
        features.bear_cv[i] = normalize_cv(bear_result.intervals_cv);

        features.max_drawdown[i] = normalize_drawdown(bull_result.max_drawdown);
        features.max_runup[i] = normalize_runup(bear_result.max_runup);
    }

    features
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_trending_nav(n: usize, trend: f64) -> Vec<f64> {
        let mut nav = Vec::with_capacity(n);
        let mut value = 1.0;
        for _ in 0..n {
            nav.push(value);
            value *= 1.0 + trend;
        }
        nav
    }

    fn generate_volatile_nav(n: usize, seed: u64) -> Vec<f64> {
        // Simple deterministic pseudo-random using LCG
        let mut state = seed;
        let mut nav = Vec::with_capacity(n);
        let mut value = 1.0;
        for _ in 0..n {
            nav.push(value);
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let rand = ((state >> 33) as f64) / (u32::MAX as f64) - 0.5;
            value *= 1.0 + rand * 0.02;
            value = value.max(0.01); // Prevent negative NAV
        }
        nav
    }

    #[test]
    fn test_rolling_ith_length() {
        let nav = generate_trending_nav(100, 0.001);
        let features = compute_rolling_ith(&nav, 20);

        assert_eq!(features.bull_epoch_density.len(), 100);
        assert_eq!(features.bear_epoch_density.len(), 100);
        assert_eq!(features.bull_excess_gain.len(), 100);
        assert_eq!(features.bear_excess_gain.len(), 100);
        assert_eq!(features.bull_cv.len(), 100);
        assert_eq!(features.bear_cv.len(), 100);
        assert_eq!(features.max_drawdown.len(), 100);
        assert_eq!(features.max_runup.len(), 100);
    }

    #[test]
    fn test_rolling_ith_first_values_nan() {
        let nav = generate_trending_nav(100, 0.001);
        let lookback = 20;
        let features = compute_rolling_ith(&nav, lookback);

        // First lookback-1 values should be NaN
        for i in 0..(lookback - 1) {
            assert!(
                features.bull_epoch_density[i].is_nan(),
                "Expected NaN at index {}",
                i
            );
        }

        // Value at lookback-1 should be valid
        assert!(
            !features.bull_epoch_density[lookback - 1].is_nan(),
            "Expected valid value at index {}",
            lookback - 1
        );
    }

    #[test]
    fn test_rolling_ith_bounded() {
        let nav = generate_volatile_nav(200, 12345);
        let features = compute_rolling_ith(&nav, 50);

        // Check all valid values are bounded [0, 1]
        for i in 49..200 {
            let check_bounded = |name: &str, val: f64| {
                assert!(
                    val >= 0.0 && val <= 1.0,
                    "{} at index {} = {} is not bounded",
                    name,
                    i,
                    val
                );
            };

            if !features.bull_epoch_density[i].is_nan() {
                check_bounded("bull_epoch_density", features.bull_epoch_density[i]);
            }
            if !features.bear_epoch_density[i].is_nan() {
                check_bounded("bear_epoch_density", features.bear_epoch_density[i]);
            }
            if !features.bull_excess_gain[i].is_nan() {
                check_bounded("bull_excess_gain", features.bull_excess_gain[i]);
            }
            if !features.bear_excess_gain[i].is_nan() {
                check_bounded("bear_excess_gain", features.bear_excess_gain[i]);
            }
            if !features.bull_cv[i].is_nan() {
                check_bounded("bull_cv", features.bull_cv[i]);
            }
            if !features.bear_cv[i].is_nan() {
                check_bounded("bear_cv", features.bear_cv[i]);
            }
            if !features.max_drawdown[i].is_nan() {
                check_bounded("max_drawdown", features.max_drawdown[i]);
            }
            if !features.max_runup[i].is_nan() {
                check_bounded("max_runup", features.max_runup[i]);
            }
        }
    }

    #[test]
    fn test_rolling_ith_uptrend_characteristics() {
        // Strong uptrend should have low drawdown
        let nav = generate_trending_nav(100, 0.01);
        let features = compute_rolling_ith(&nav, 20);

        // In pure uptrend, max_drawdown should be 0 or very small
        for i in 19..100 {
            assert!(
                features.max_drawdown[i] < 0.01,
                "Expected low drawdown in uptrend at index {}",
                i
            );
        }
    }

    #[test]
    fn test_rolling_ith_downtrend_characteristics() {
        // Strong downtrend should have low runup
        let nav = generate_trending_nav(100, -0.01);
        let features = compute_rolling_ith(&nav, 20);

        // In pure downtrend, max_runup should be 0 or very small
        for i in 19..100 {
            assert!(
                features.max_runup[i] < 0.01,
                "Expected low runup in downtrend at index {}",
                i
            );
        }
    }

    #[test]
    #[should_panic(expected = "lookback must be positive")]
    fn test_rolling_ith_zero_lookback() {
        let nav = generate_trending_nav(100, 0.001);
        compute_rolling_ith(&nav, 0);
    }

    #[test]
    #[should_panic(expected = "lookback cannot exceed NAV length")]
    fn test_rolling_ith_lookback_too_large() {
        let nav = generate_trending_nav(50, 0.001);
        compute_rolling_ith(&nav, 100);
    }

    #[test]
    fn test_rolling_ith_minimum_lookback() {
        // Lookback of 1 should work (edge case)
        let nav = vec![1.0, 1.01, 1.02];
        let features = compute_rolling_ith(&nav, 1);

        // All values should be valid (not NaN)
        assert!(!features.bull_epoch_density[0].is_nan());
        assert!(!features.bull_epoch_density[1].is_nan());
        assert!(!features.bull_epoch_density[2].is_nan());
    }

    #[test]
    fn test_rolling_ith_exact_lookback() {
        // Lookback equals NAV length
        let nav = vec![1.0, 1.01, 1.02, 1.015, 1.03];
        let features = compute_rolling_ith(&nav, 5);

        // First 4 should be NaN, last should be valid
        assert!(features.bull_epoch_density[0].is_nan());
        assert!(features.bull_epoch_density[3].is_nan());
        assert!(!features.bull_epoch_density[4].is_nan());
    }
}
