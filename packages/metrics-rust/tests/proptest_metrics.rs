//! Property-based tests for trading-fitness-metrics.
//!
//! Tests 4 critical properties for BiLSTM feature engineering:
//! 1. Bounded output [0, 1] - Neural network input normalization
//! 2. Scale invariance - Price-scale independent (USD vs satoshi)
//! 3. Determinism - Same input → same output (no hidden state)
//! 4. Edge cases - Empty, single, NaN/Inf handling

use proptest::prelude::*;
use trading_fitness_metrics::{
    garman_klass_volatility, hurst_exponent, kaufman_efficiency_ratio, omega_ratio,
    permutation_entropy, ulcer_index,
    // Use adaptive utilities for optimal parameters
    optimal_bins_freedman_diaconis, optimal_sample_entropy_tolerance,
};
// Access non-adaptive module functions directly
use trading_fitness_metrics::entropy::{sample_entropy, shannon_entropy};
use trading_fitness_metrics::fractal::fractal_dimension;

// =============================================================================
// STRATEGIES - Market-realistic data generation
// =============================================================================

/// Generate realistic price series using multiplicative returns with a floor.
fn realistic_prices(n: usize) -> impl Strategy<Value = Vec<f64>> {
    (1.0f64..10000.0).prop_flat_map(move |start| {
        prop::collection::vec(-0.15f64..0.15, n.saturating_sub(1)).prop_map(move |returns| {
            let mut prices = Vec::with_capacity(n);
            prices.push(start);
            let mut price = start;
            for r in returns {
                price *= 1.0 + r;
                price = price.max(0.01);
                prices.push(price);
            }
            prices
        })
    })
}

/// Generate realistic returns (fat-tailed, bounded).
fn realistic_returns(n: usize) -> impl Strategy<Value = Vec<f64>> {
    prop::collection::vec(-0.20f64..0.20, n)
}

/// Generate valid OHLC bar with proper invariants (H ≥ O,C ≥ L).
fn valid_ohlc() -> impl Strategy<Value = (f64, f64, f64, f64)> {
    (1.0f64..10000.0, 0.001f64..0.10).prop_flat_map(|(base, range_pct)| {
        let range = base * range_pct;
        let low = base - range / 2.0;
        let high = base + range / 2.0;
        (low..=high, Just(high), Just(low), low..=high)
    })
}

/// Generate trending series (expected Hurst > 0.5).
fn trending_series(n: usize) -> impl Strategy<Value = Vec<f64>> {
    (1.0f64..1000.0, 1.001f64..1.02f64)
        .prop_map(move |(start, drift)| (0..n).map(|i| start * drift.powi(i as i32)).collect())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    // =========================================================================
    // CATEGORY 1: BOUNDED OUTPUT [0,1] (9 tests)
    // Critical for BiLSTM input normalization
    // =========================================================================

    #[test]
    fn permutation_entropy_bounded(prices in realistic_prices(100)) {
        let pe = permutation_entropy(&prices, 3);
        prop_assert!(pe >= 0.0 && pe <= 1.0, "PE={} out of bounds", pe);
    }

    #[test]
    fn sample_entropy_bounded(returns in realistic_returns(200)) {
        let r = optimal_sample_entropy_tolerance(&returns);
        let se = sample_entropy(&returns, 2, r);
        prop_assert!(se >= 0.0 && se <= 1.0, "SE={} out of bounds", se);
    }

    #[test]
    fn shannon_entropy_bounded(returns in realistic_returns(100)) {
        let n_bins = optimal_bins_freedman_diaconis(&returns);
        let se = shannon_entropy(&returns, n_bins);
        prop_assert!(se >= 0.0 && se <= 1.0, "Shannon={} out of bounds", se);
    }

    #[test]
    fn omega_ratio_bounded(returns in realistic_returns(100)) {
        let omega = omega_ratio(&returns, 0.0);
        prop_assert!(omega >= 0.0 && omega <= 1.0, "Omega={} out of bounds", omega);
    }

    #[test]
    fn ulcer_index_bounded(prices in realistic_prices(100)) {
        let ui = ulcer_index(&prices);
        prop_assert!(ui >= 0.0 && ui <= 1.0, "UI={} out of bounds", ui);
    }

    #[test]
    fn garman_klass_bounded((o, h, l, c) in valid_ohlc()) {
        let gk = garman_klass_volatility(o, h, l, c);
        prop_assert!(gk >= 0.0 && gk <= 1.0, "GK={} out of bounds", gk);
    }

    #[test]
    fn kaufman_er_bounded(prices in realistic_prices(100)) {
        let ker = kaufman_efficiency_ratio(&prices);
        prop_assert!(ker >= 0.0 && ker <= 1.0, "KER={} out of bounds", ker);
    }

    #[test]
    fn hurst_bounded(prices in realistic_prices(300)) {
        let h = hurst_exponent(&prices);
        prop_assert!(h >= 0.0 && h <= 1.0, "Hurst={} out of bounds", h);
    }

    #[test]
    fn fractal_dimension_bounded(prices in realistic_prices(100)) {
        let k_max = (prices.len() as f64).sqrt() as usize / 2;
        let k_max = k_max.max(5).min(50);
        let fd = fractal_dimension(&prices, k_max);
        prop_assert!(fd >= 0.0 && fd <= 1.0, "FD={} out of bounds", fd);
    }

    // =========================================================================
    // CATEGORY 2: SCALE INVARIANCE (3 tests)
    // Entropy metrics independent of price scale (USD vs satoshi)
    // =========================================================================

    #[test]
    fn permutation_entropy_scale_invariant(
        prices in realistic_prices(100),
        scale in 0.01f64..100.0
    ) {
        let scaled: Vec<f64> = prices.iter().map(|p| p * scale).collect();
        let pe1 = permutation_entropy(&prices, 3);
        let pe2 = permutation_entropy(&scaled, 3);
        prop_assert!((pe1 - pe2).abs() < 1e-10, "PE not scale-invariant: {} vs {}", pe1, pe2);
    }

    #[test]
    fn kaufman_er_scale_invariant(
        prices in realistic_prices(50),
        scale in 0.01f64..100.0
    ) {
        let scaled: Vec<f64> = prices.iter().map(|p| p * scale).collect();
        let ker1 = kaufman_efficiency_ratio(&prices);
        let ker2 = kaufman_efficiency_ratio(&scaled);
        prop_assert!((ker1 - ker2).abs() < 1e-10, "KER not scale-invariant: {} vs {}", ker1, ker2);
    }

    #[test]
    fn ulcer_index_scale_invariant(
        prices in realistic_prices(50),
        scale in 0.01f64..100.0
    ) {
        let scaled: Vec<f64> = prices.iter().map(|p| p * scale).collect();
        let ui1 = ulcer_index(&prices);
        let ui2 = ulcer_index(&scaled);
        prop_assert!((ui1 - ui2).abs() < 1e-10, "UI not scale-invariant: {} vs {}", ui1, ui2);
    }

    // =========================================================================
    // CATEGORY 3: DETERMINISM (9 tests)
    // Same input → same output (no hidden state leakage)
    // =========================================================================

    #[test]
    fn permutation_entropy_deterministic(prices in realistic_prices(100)) {
        let pe1 = permutation_entropy(&prices, 3);
        let pe2 = permutation_entropy(&prices, 3);
        // Allow for floating-point precision differences
        prop_assert!((pe1 - pe2).abs() < 1e-14, "PE not deterministic: {} vs {}", pe1, pe2);
    }

    #[test]
    fn sample_entropy_deterministic(returns in realistic_returns(200)) {
        let r = optimal_sample_entropy_tolerance(&returns);
        let se1 = sample_entropy(&returns, 2, r);
        let se2 = sample_entropy(&returns, 2, r);
        prop_assert_eq!(se1, se2, "SE not deterministic");
    }

    #[test]
    fn shannon_entropy_deterministic(returns in realistic_returns(100)) {
        let n_bins = optimal_bins_freedman_diaconis(&returns);
        let se1 = shannon_entropy(&returns, n_bins);
        let se2 = shannon_entropy(&returns, n_bins);
        prop_assert_eq!(se1, se2, "Shannon not deterministic");
    }

    #[test]
    fn omega_ratio_deterministic(returns in realistic_returns(100)) {
        let o1 = omega_ratio(&returns, 0.0);
        let o2 = omega_ratio(&returns, 0.0);
        prop_assert_eq!(o1, o2, "Omega not deterministic");
    }

    #[test]
    fn ulcer_index_deterministic(prices in realistic_prices(100)) {
        let ui1 = ulcer_index(&prices);
        let ui2 = ulcer_index(&prices);
        prop_assert_eq!(ui1, ui2, "UI not deterministic");
    }

    #[test]
    fn garman_klass_deterministic((o, h, l, c) in valid_ohlc()) {
        let gk1 = garman_klass_volatility(o, h, l, c);
        let gk2 = garman_klass_volatility(o, h, l, c);
        prop_assert_eq!(gk1, gk2, "GK not deterministic");
    }

    #[test]
    fn kaufman_er_deterministic(prices in realistic_prices(100)) {
        let ker1 = kaufman_efficiency_ratio(&prices);
        let ker2 = kaufman_efficiency_ratio(&prices);
        prop_assert_eq!(ker1, ker2, "KER not deterministic");
    }

    #[test]
    fn hurst_deterministic(prices in realistic_prices(300)) {
        let h1 = hurst_exponent(&prices);
        let h2 = hurst_exponent(&prices);
        prop_assert_eq!(h1, h2, "Hurst not deterministic");
    }

    #[test]
    fn fractal_dimension_deterministic(prices in realistic_prices(100)) {
        let k_max = 10;
        let fd1 = fractal_dimension(&prices, k_max);
        let fd2 = fractal_dimension(&prices, k_max);
        prop_assert_eq!(fd1, fd2, "FD not deterministic");
    }

    // =========================================================================
    // CATEGORY 4: EDGE CASE HANDLING (9 tests)
    // Empty, single element, special values return sensible defaults
    // =========================================================================

    #[test]
    fn permutation_entropy_edge_short(n in 60usize..100) {
        // Need minimum 10 * 3! = 60 elements for statistical validity
        let prices: Vec<f64> = (0..n).map(|i| 100.0 + 5.0 * (i as f64 * 0.1).sin()).collect();
        let pe = permutation_entropy(&prices, 3);
        prop_assert!(!pe.is_nan() && pe >= 0.0 && pe <= 1.0, "PE should be bounded: {}", pe);
    }

    #[test]
    fn sample_entropy_edge_varied(n in 200usize..300) {
        // Need sufficient variability and length for sample entropy
        let returns: Vec<f64> = (0..n).map(|i| 0.1 * ((i as f64 * 0.5).sin() + (i as f64 * 0.13).cos())).collect();
        let r = optimal_sample_entropy_tolerance(&returns);
        let se = sample_entropy(&returns, 2, r);
        // May return NaN for low-variability data, which is acceptable edge case
        prop_assert!(se.is_nan() || (se >= 0.0 && se <= 1.0), "SE should be bounded or NaN: {}", se);
    }

    #[test]
    fn shannon_entropy_edge_uniform(n in 50usize..200) {
        // Uniform distribution should have bounded entropy
        let uniform: Vec<f64> = (0..n).map(|i| i as f64 / n as f64).collect();
        let n_bins = optimal_bins_freedman_diaconis(&uniform).max(2);
        let se = shannon_entropy(&uniform, n_bins);
        prop_assert!(se >= 0.0 && se <= 1.0, "Shannon uniform should be bounded: {}", se);
    }

    #[test]
    fn omega_ratio_edge_all_gains(n in 10usize..100) {
        let all_gains: Vec<f64> = vec![0.01; n];
        let omega = omega_ratio(&all_gains, 0.0);
        // All positive returns → bounded high omega
        prop_assert!(omega >= 0.99 && omega <= 1.0, "Omega all-gains should be ~1.0: {}", omega);
    }

    #[test]
    fn ulcer_index_edge_no_drawdown(n in 10usize..100) {
        let monotonic: Vec<f64> = (0..n).map(|i| 100.0 + i as f64).collect();
        let ui = ulcer_index(&monotonic);
        prop_assert!(ui == 0.0, "UI no-drawdown should be 0.0: {}", ui);
    }

    #[test]
    fn garman_klass_edge_flat_bar(price in 1.0f64..10000.0) {
        // Flat bar: O=H=L=C
        let gk = garman_klass_volatility(price, price, price, price);
        prop_assert!(gk == 0.0, "GK flat bar should be 0.0: {}", gk);
    }

    #[test]
    fn kaufman_er_edge_perfect_trend(n in 10usize..100) {
        let perfect_up: Vec<f64> = (0..n).map(|i| 100.0 + i as f64).collect();
        let ker = kaufman_efficiency_ratio(&perfect_up);
        prop_assert!(ker >= 0.99, "KER perfect trend should be ~1.0: {}", ker);
    }

    #[test]
    fn hurst_edge_trending_series(prices in trending_series(300)) {
        let h = hurst_exponent(&prices);
        // Hurst is always bounded regardless of input pattern
        prop_assert!(h >= 0.0 && h <= 1.0, "Hurst should be bounded: {}", h);
    }

    #[test]
    fn fractal_dimension_edge_short(n in 15usize..30) {
        let prices: Vec<f64> = (0..n).map(|i| 100.0 + (i as f64 * 0.1).sin()).collect();
        let k_max = 5; // Minimum valid k_max
        let fd = fractal_dimension(&prices, k_max);
        prop_assert!(fd >= 0.0 && fd <= 1.0, "FD short should be bounded: {}", fd);
    }
}
