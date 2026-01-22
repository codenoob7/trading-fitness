//! Market-realistic proptest strategies for metrics testing.
//!
//! These strategies generate price series and returns with realistic market properties
//! for property-based testing of trading metrics.
#![cfg(test)]

use proptest::prelude::*;
use proptest::strategy::ValueTree;

/// Generate realistic price series using multiplicative returns with a floor.
///
/// Properties:
/// - Log-normal style evolution (multiplicative returns)
/// - Price floor at 0.01 to prevent negative/zero prices
/// - Returns bounded to ±15% per step (extreme but realistic for crypto)
///
/// # Arguments
/// * `n` - Number of price points to generate
pub fn realistic_prices(n: usize) -> impl Strategy<Value = Vec<f64>> {
    (1.0f64..10000.0).prop_flat_map(move |start| {
        prop::collection::vec(-0.15f64..0.15, n.saturating_sub(1))
            .prop_map(move |returns| {
                let mut prices = Vec::with_capacity(n);
                prices.push(start);
                let mut price = start;
                for r in returns {
                    price *= 1.0 + r;
                    price = price.max(0.01); // Floor to prevent degenerate cases
                    prices.push(price);
                }
                prices
            })
    })
}

/// Generate valid OHLC bar with proper invariants (H ≥ O,C ≥ L).
///
/// Properties:
/// - High is always the maximum
/// - Low is always the minimum
/// - Open and Close are between High and Low
pub fn valid_ohlc() -> impl Strategy<Value = (f64, f64, f64, f64)> {
    (1.0f64..10000.0, 0.001f64..0.10).prop_flat_map(|(base, range_pct)| {
        let range = base * range_pct;
        let low = base - range / 2.0;
        let high = base + range / 2.0;
        (
            low..=high,         // Open
            Just(high),         // High
            Just(low),          // Low
            low..=high,         // Close
        )
    })
}

/// Generate trending series (expected Hurst > 0.5).
///
/// Creates a monotonically increasing series with geometric drift.
///
/// # Arguments
/// * `n` - Number of price points to generate
pub fn trending_series(n: usize) -> impl Strategy<Value = Vec<f64>> {
    (1.0f64..1000.0, 1.001f64..1.02f64).prop_map(move |(start, drift)| {
        (0..n).map(|i| start * drift.powi(i as i32)).collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::test_runner::TestRunner;

    #[test]
    fn realistic_prices_generates_correct_length() {
        let mut runner = TestRunner::default();
        let strategy = realistic_prices(100);

        for _ in 0..10 {
            let prices = strategy.new_tree(&mut runner).unwrap().current();
            assert_eq!(prices.len(), 100);
            assert!(prices.iter().all(|&p| p >= 0.01));
        }
    }

    #[test]
    fn valid_ohlc_maintains_invariants() {
        let mut runner = TestRunner::default();
        let strategy = valid_ohlc();

        for _ in 0..100 {
            let (o, h, l, c) = strategy.new_tree(&mut runner).unwrap().current();
            assert!(h >= o, "High {} must be >= Open {}", h, o);
            assert!(h >= c, "High {} must be >= Close {}", h, c);
            assert!(l <= o, "Low {} must be <= Open {}", l, o);
            assert!(l <= c, "Low {} must be <= Close {}", l, c);
        }
    }

    #[test]
    fn trending_series_is_monotonic() {
        let mut runner = TestRunner::default();
        let strategy = trending_series(50);

        for _ in 0..10 {
            let prices = strategy.new_tree(&mut runner).unwrap().current();
            for window in prices.windows(2) {
                assert!(
                    window[1] >= window[0],
                    "Trending series should be monotonic"
                );
            }
        }
    }
}
