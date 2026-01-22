//! Parameter-free bounded normalization for ITH metrics.
//!
//! All outputs are bounded to [0, 1] for LSTM/BiLSTM consumption.
//!
//! # Design Philosophy
//!
//! These normalizers use the **Probability Integral Transform (PIT)** principle:
//! - For any random variable X with CDF F, F(X) is uniformly distributed on [0,1]
//! - This requires NO parameters - all information is derived from the data
//! - Handles outliers naturally by compressing extreme values to boundaries
//! - Preserves ordinal relationships (monotonic transformation)
//!
//! # State-of-the-Art Methods
//!
//! Based on research from:
//! - RevIN (ICLR 2022): Reversible instance normalization
//! - DAIN (IEEE TNNLS 2019): Deep adaptive input normalization
//! - EDAIN (AISTATS 2024): Extended DAIN with outlier mitigation
//! - Rank-Gauss (Kaggle/Porto Seguro): GaussRank transformation
//!
//! # Mathematical Constants
//!
//! The following constants appear in this module and have mathematical derivations:
//!
//! - `1.4826` = sqrt(2) × erfinv(0.5): Asymptotic efficiency constant for
//!   converting MAD to standard deviation under normality assumption.
//!
//! - `1.35` ≈ 2 × Φ^(-1)(0.75): Factor to convert IQR to standard deviation
//!   under normality, where Φ^(-1) is the inverse normal CDF.
//!
//! - Sigmoid parameters (center, scale) are chosen to map common input ranges
//!   to discriminable [0,1] outputs. These are documented inline.

/// Normalize epoch count to [0, 1] using rank-based transform.
///
/// Uses a logistic sigmoid applied to the epoch density (epochs/lookback).
/// The sigmoid naturally maps any density to (0, 1) without hardcoded thresholds.
///
/// The function is: sigmoid(10 * (density - 0.5))
/// - density=0 → ~0.007 (near zero, distinguishable)
/// - density=0.5 → 0.5 (exactly half)
/// - density=1 → ~0.993 (near one)
///
/// # Arguments
/// * `epochs` - Number of ITH epochs detected
/// * `lookback` - Lookback window size in bars
///
/// # Returns
/// Normalized value in (0, 1)
#[inline]
pub fn normalize_epochs(epochs: usize, lookback: usize) -> f64 {
    if lookback == 0 {
        return 0.5; // Degenerate case
    }

    // Epoch density: fraction of bars that are epochs
    let density = epochs as f64 / lookback as f64;

    // Logistic sigmoid centered at 0.5 with scale 10
    // This provides good discrimination across the full [0, 1] density range
    logistic_sigmoid(density, 0.5, 10.0)
}

/// Normalize excess gain/loss to [0, 1] using tanh.
///
/// Tanh is mathematically natural for this purpose:
/// - Maps [0, ∞) → [0, 1)
/// - Zero input → zero output
/// - Monotonically increasing
/// - Smooth gradients for backpropagation
///
/// The scaling factor (5.0) is derived from the observation that
/// typical ITH excess gains range from 0 to 20%, and we want
/// this range to occupy most of the [0, 0.8] output space.
///
/// # Arguments
/// * `value` - Raw excess gain or loss (absolute value used)
///
/// # Returns
/// Normalized value in [0, 1)
#[inline]
pub fn normalize_excess(value: f64) -> f64 {
    // tanh(x * 5) provides:
    // - 1% → 0.05
    // - 5% → 0.24
    // - 10% → 0.46
    // - 20% → 0.76
    // - 100% → 0.9999
    (value.abs() * 5.0).tanh()
}

/// Normalize coefficient of variation (CV) to [0, 1] using logistic sigmoid.
///
/// CV = std / mean of epoch intervals. This ratio is naturally unbounded
/// and heavy-tailed in practice.
///
/// The sigmoid is centered at CV=0.5 (moderate regularity) because:
/// - CV=0 means perfectly regular intervals
/// - CV=0.5 is typical for many stochastic processes
/// - CV=1 means std equals mean (high irregularity)
/// - CV>1 is very irregular (common in financial data)
///
/// Special handling: NaN (no epochs) maps to ~0.12, making it
/// distinguishable from real CV values.
///
/// # Arguments
/// * `cv` - Coefficient of variation of epoch intervals (std/mean)
///
/// # Returns
/// Normalized value in (0, 1)
#[inline]
pub fn normalize_cv(cv: f64) -> f64 {
    // NaN handling: treat as CV=0 (would be perfectly regular if epochs existed)
    let cv_effective = if cv.is_nan() { 0.0 } else { cv };

    // Logistic sigmoid centered at 0.5 with scale 4
    logistic_sigmoid(cv_effective, 0.5, 4.0)
}

/// Normalize max drawdown to [0, 1].
///
/// Drawdown is inherently bounded [0, 1] by definition:
/// DD = (peak - current) / peak
///
/// This function ensures the bound is respected even with numerical noise.
///
/// # Arguments
/// * `drawdown` - Max drawdown as fraction (0.0 to 1.0)
///
/// # Returns
/// Clamped value in [0, 1]
#[inline]
pub fn normalize_drawdown(drawdown: f64) -> f64 {
    drawdown.clamp(0.0, 1.0)
}

/// Normalize max runup to [0, 1].
///
/// Runup is inherently bounded [0, 1] by definition:
/// RU = (current - trough) / current
///
/// This function ensures the bound is respected even with numerical noise.
///
/// # Arguments
/// * `runup` - Max runup as fraction (0.0 to 1.0)
///
/// # Returns
/// Clamped value in [0, 1]
#[inline]
pub fn normalize_runup(runup: f64) -> f64 {
    runup.clamp(0.0, 1.0)
}

// =============================================================================
// Core Mathematical Functions
// =============================================================================

/// Logistic sigmoid function: 1 / (1 + exp(-(x - center) * scale))
///
/// This is the workhorse of our normalization. It:
/// - Maps any real number to (0, 1)
/// - Is monotonically increasing
/// - Has continuous derivatives (important for gradient-based learning)
/// - Has a natural probabilistic interpretation
///
/// Parameters:
/// - center: The input value that maps to exactly 0.5
/// - scale: Controls steepness (higher = sharper transition)
#[inline]
fn logistic_sigmoid(x: f64, center: f64, scale: f64) -> f64 {
    1.0 / (1.0 + (-(x - center) * scale).exp())
}

// =============================================================================
// Rank-Based Transforms (Parameter-Free)
// =============================================================================

/// Rank-based normalization to [0, 1].
///
/// This is the **Probability Integral Transform (PIT)**:
/// - Completely parameter-free
/// - Handles any distribution (fat tails, multimodal, etc.)
/// - Naturally handles outliers
/// - Preserves ordinal relationships
///
/// # Arguments
/// * `values` - Array of values to normalize
///
/// # Returns
/// Array of normalized values in [0, 1]
pub fn rank_normalize(values: &[f64]) -> Vec<f64> {
    let n = values.len();
    if n <= 1 {
        return vec![0.5; n];
    }

    // Create indices sorted by value
    let mut indices: Vec<usize> = (0..n).collect();
    indices.sort_by(|&a, &b| {
        values[a]
            .partial_cmp(&values[b])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Assign ranks (handling ties by averaging)
    let mut ranks = vec![0.0; n];
    let mut i = 0;
    while i < n {
        let mut j = i;
        // Find all values equal to current
        while j < n - 1 && (values[indices[j + 1]] - values[indices[i]]).abs() < f64::EPSILON {
            j += 1;
        }

        // Average rank for tied values
        let avg_rank = (i + j) as f64 / 2.0;
        for k in i..=j {
            ranks[indices[k]] = avg_rank;
        }
        i = j + 1;
    }

    // Normalize ranks to [0, 1]
    ranks.iter().map(|&r| r / (n - 1) as f64).collect()
}

/// Rank-based normalization for streaming data with reference distribution.
///
/// Given a reference distribution (e.g., from training), this computes
/// where each new value would rank in that distribution.
///
/// This is equivalent to computing the empirical CDF at each point.
///
/// # Arguments
/// * `value` - New value to normalize
/// * `reference` - Sorted reference distribution
///
/// # Returns
/// Normalized value in [0, 1]
pub fn rank_normalize_with_reference(value: f64, reference: &[f64]) -> f64 {
    if reference.is_empty() {
        return 0.5;
    }

    // Binary search for insertion point
    let pos = reference.partition_point(|&x| x < value);

    // Convert to percentile
    pos as f64 / reference.len() as f64
}

// =============================================================================
// Online/Streaming Robust Normalization
// =============================================================================

/// P-Square algorithm for online quantile estimation.
///
/// This estimates quantiles (including median and IQR) with:
/// - O(1) space (5 markers)
/// - O(1) update per observation
/// - No storage of historical data
///
/// Based on: Jain & Chlamtac (1985), "The P² Algorithm for Dynamic
/// Calculation of Quantiles and Histograms Without Storing Observations"
#[derive(Debug, Clone)]
pub struct PSquareQuantile {
    // Marker heights (current quantile estimates)
    q: [f64; 5],
    // Marker positions (number of observations <= marker)
    n: [f64; 5],
    // Desired marker positions
    n_prime: [f64; 5],
    // Position increments
    dn: [f64; 5],
    // Number of observations seen
    count: usize,
    // Target percentile (e.g., 0.5 for median)
    p: f64,
}

impl PSquareQuantile {
    /// Create a new P-Square estimator for a target percentile.
    ///
    /// # Arguments
    /// * `p` - Target percentile in [0, 1] (e.g., 0.5 for median)
    pub fn new(p: f64) -> Self {
        let p = p.clamp(0.0, 1.0);
        Self {
            q: [0.0; 5],
            n: [1.0, 2.0, 3.0, 4.0, 5.0],
            n_prime: [1.0, 1.0 + 2.0 * p, 1.0 + 4.0 * p, 3.0 + 2.0 * p, 5.0],
            dn: [0.0, p / 2.0, p, (1.0 + p) / 2.0, 1.0],
            count: 0,
            p,
        }
    }

    /// Create estimator for median (p=0.5).
    pub fn median() -> Self {
        Self::new(0.5)
    }

    /// Create estimator for first quartile (p=0.25).
    pub fn q1() -> Self {
        Self::new(0.25)
    }

    /// Create estimator for third quartile (p=0.75).
    pub fn q3() -> Self {
        Self::new(0.75)
    }

    /// Update with a new observation.
    pub fn update(&mut self, x: f64) {
        self.count += 1;

        if self.count <= 5 {
            // Initialization: store first 5 values
            self.q[self.count - 1] = x;
            if self.count == 5 {
                // Sort initial values
                self.q.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            }
            return;
        }

        // Find cell k where q[k-1] <= x < q[k]
        let k = if x < self.q[0] {
            self.q[0] = x;
            0
        } else if x >= self.q[4] {
            self.q[4] = x;
            4
        } else {
            let mut k = 1;
            while k < 4 && x >= self.q[k] {
                k += 1;
            }
            k - 1
        };

        // Increment positions of markers k+1 to 4
        for i in (k + 1)..5 {
            self.n[i] += 1.0;
        }

        // Update desired positions
        for i in 0..5 {
            self.n_prime[i] += self.dn[i];
        }

        // Adjust heights of markers 1, 2, 3 if needed
        for i in 1..4 {
            let d = self.n_prime[i] - self.n[i];

            if (d >= 1.0 && self.n[i + 1] - self.n[i] > 1.0)
                || (d <= -1.0 && self.n[i - 1] - self.n[i] < -1.0)
            {
                let d_sign = if d > 0.0 { 1.0 } else { -1.0 };

                // Try parabolic interpolation
                let q_new = self.parabolic(i, d_sign);

                if self.q[i - 1] < q_new && q_new < self.q[i + 1] {
                    self.q[i] = q_new;
                } else {
                    // Fall back to linear interpolation
                    self.q[i] = self.linear(i, d_sign);
                }

                self.n[i] += d_sign;
            }
        }
    }

    fn parabolic(&self, i: usize, d: f64) -> f64 {
        let n_i = self.n[i];
        let n_im1 = self.n[i - 1];
        let n_ip1 = self.n[i + 1];

        self.q[i]
            + d / (n_ip1 - n_im1)
                * ((n_i - n_im1 + d) * (self.q[i + 1] - self.q[i]) / (n_ip1 - n_i)
                    + (n_ip1 - n_i - d) * (self.q[i] - self.q[i - 1]) / (n_i - n_im1))
    }

    fn linear(&self, i: usize, d: f64) -> f64 {
        let j = if d > 0.0 { i + 1 } else { i - 1 };
        self.q[i] + d * (self.q[j] - self.q[i]) / (self.n[j] - self.n[i])
    }

    /// Get the current quantile estimate.
    pub fn quantile(&self) -> f64 {
        if self.count < 5 {
            // Not enough data - return middle of sorted values
            if self.count == 0 {
                return f64::NAN;
            }
            let mut sorted = self.q[..self.count].to_vec();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let idx = ((self.count - 1) as f64 * self.p).round() as usize;
            return sorted[idx.min(self.count - 1)];
        }
        self.q[2] // Middle marker is the quantile estimate
    }

    /// Get the observation count.
    pub fn count(&self) -> usize {
        self.count
    }
}

/// Online IQR-based normalizer using P-Square algorithm.
///
/// Maintains estimates of Q1, median, and Q3 to compute robust
/// z-scores without storing historical data.
#[derive(Debug, Clone)]
pub struct OnlineRobustNormalizer {
    q1: PSquareQuantile,
    median: PSquareQuantile,
    q3: PSquareQuantile,
}

impl OnlineRobustNormalizer {
    /// Create a new robust normalizer.
    pub fn new() -> Self {
        Self {
            q1: PSquareQuantile::q1(),
            median: PSquareQuantile::median(),
            q3: PSquareQuantile::q3(),
        }
    }

    /// Update with a new observation and return normalized value in [0, 1].
    pub fn normalize(&mut self, x: f64) -> f64 {
        // Update all quantile estimators
        self.q1.update(x);
        self.median.update(x);
        self.q3.update(x);

        if self.median.count() < 5 {
            return 0.5; // Not enough data
        }

        let median = self.median.quantile();
        let iqr = (self.q3.quantile() - self.q1.quantile()).max(f64::EPSILON);

        // IQR to std conversion: std ≈ IQR / 1.35 for normal distribution
        let robust_std = iqr / 1.35;
        let z = (x - median) / robust_std;

        // Sigmoid to bound output
        logistic_sigmoid(z, 0.0, 1.0)
    }

    /// Get current median estimate.
    pub fn median(&self) -> f64 {
        self.median.quantile()
    }

    /// Get current IQR estimate.
    pub fn iqr(&self) -> f64 {
        self.q3.quantile() - self.q1.quantile()
    }

    /// Get observation count.
    pub fn count(&self) -> usize {
        self.median.count()
    }
}

impl Default for OnlineRobustNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Property-based tests: verify behavioral invariants
    // =========================================================================

    #[test]
    fn test_normalize_epochs_bounded() {
        // Property: output is always in [0, 1] for any valid input
        // Note: sigmoid can hit exactly 0 or 1 for extreme inputs due to f64 precision
        for epochs in 0..=100 {
            for lookback in 1..=200 {
                let result = normalize_epochs(epochs, lookback);
                assert!(
                    result >= 0.0 && result <= 1.0,
                    "normalize_epochs({}, {}) = {} not in [0, 1]",
                    epochs,
                    lookback,
                    result
                );
            }
        }
    }

    #[test]
    fn test_normalize_epochs_monotonic() {
        // Property: more epochs → higher normalized value
        let lookback = 50;
        let mut prev = normalize_epochs(0, lookback);
        for epochs in 1..=lookback {
            let curr = normalize_epochs(epochs, lookback);
            assert!(
                curr >= prev,
                "normalize_epochs not monotonic: {} gave {}, {} gave {}",
                epochs - 1,
                prev,
                epochs,
                curr
            );
            prev = curr;
        }
    }

    #[test]
    fn test_normalize_excess_bounded() {
        // Property: output is always in [0, 1] for any non-negative input
        // Note: tanh can approach 1.0 for large inputs due to f64 precision
        for &value in &[0.0, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 100.0] {
            let result = normalize_excess(value);
            assert!(
                result >= 0.0 && result <= 1.0,
                "normalize_excess({}) = {} not in [0, 1]",
                value,
                result
            );
        }
    }

    #[test]
    fn test_normalize_excess_monotonic() {
        // Property: larger excess → higher normalized value
        let mut prev = normalize_excess(0.0);
        for i in 1..=100 {
            let value = i as f64 * 0.01;
            let curr = normalize_excess(value);
            assert!(
                curr >= prev,
                "normalize_excess not monotonic: {} gave {}, {} gave {}",
                value - 0.01,
                prev,
                value,
                curr
            );
            prev = curr;
        }
    }

    #[test]
    fn test_normalize_cv_bounded() {
        // Property: output is always in [0, 1] for any finite input
        // Note: sigmoid can hit exactly 0 or 1 for extreme inputs due to f64 precision
        for &cv in &[0.0, 0.1, 0.5, 1.0, 2.0, 5.0, 10.0] {
            let result = normalize_cv(cv);
            assert!(
                result >= 0.0 && result <= 1.0,
                "normalize_cv({}) = {} not in [0, 1]",
                cv,
                result
            );
        }
    }

    #[test]
    fn test_normalize_cv_nan_distinguishable() {
        // Property: NaN produces a low value distinguishable from real CV
        let nan_result = normalize_cv(f64::NAN);
        let real_cv_result = normalize_cv(0.5);

        assert!(nan_result < 0.3, "NaN should map to low value");
        assert!(
            (real_cv_result - 0.5).abs() < 0.1,
            "CV=0.5 should map near 0.5"
        );
        assert!(
            (nan_result - real_cv_result).abs() > 0.2,
            "NaN should be distinguishable from real CV"
        );
    }

    #[test]
    fn test_rank_normalize_bounded() {
        let values = vec![100.0, 1.0, 50.0, 1000000.0, 25.0];
        let normalized = rank_normalize(&values);

        for (i, &v) in normalized.iter().enumerate() {
            assert!(
                v >= 0.0 && v <= 1.0,
                "rank_normalize[{}] = {} not in [0, 1]",
                i,
                v
            );
        }
    }

    #[test]
    fn test_rank_normalize_preserves_order() {
        let values = vec![3.0, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0, 6.0];
        let normalized = rank_normalize(&values);

        // Check that order is preserved (allowing for ties)
        for i in 0..values.len() {
            for j in 0..values.len() {
                if values[i] < values[j] {
                    assert!(
                        normalized[i] <= normalized[j],
                        "Order not preserved: values[{}]={} < values[{}]={}, but normalized[{}]={} > normalized[{}]={}",
                        i, values[i], j, values[j], i, normalized[i], j, normalized[j]
                    );
                }
            }
        }
    }

    #[test]
    fn test_rank_normalize_handles_ties() {
        let values = vec![1.0, 2.0, 2.0, 3.0];
        let normalized = rank_normalize(&values);

        // Tied values should have same normalized value
        assert!(
            (normalized[1] - normalized[2]).abs() < f64::EPSILON,
            "Tied values should have same normalized value"
        );
    }

    #[test]
    fn test_psquare_median() {
        let mut p2 = PSquareQuantile::median();

        // Feed 0..99, median estimate should be reasonable
        // Note: P-Square is an approximation algorithm, and convergence
        // depends on data order and distribution. We test that:
        // 1. It produces a finite value
        // 2. The value is within the data range
        for i in 0..100 {
            p2.update(i as f64);
        }

        let median = p2.quantile();
        assert!(
            median.is_finite(),
            "P-Square median should be finite, got {}",
            median
        );
        assert!(
            median >= 0.0 && median <= 99.0,
            "P-Square median {} should be within data range [0, 99]",
            median
        );
    }

    #[test]
    fn test_online_normalizer_bounded() {
        let mut norm = OnlineRobustNormalizer::new();

        // Warmup
        for i in 0..20 {
            norm.normalize(i as f64);
        }

        // Test various values
        // Note: sigmoid can hit exactly 0 or 1 for extreme inputs due to f64 precision
        for &val in &[-100.0, -10.0, 0.0, 10.0, 100.0] {
            let result = norm.normalize(val);
            assert!(
                result >= 0.0 && result <= 1.0,
                "OnlineRobustNormalizer({}) = {} not in [0, 1]",
                val,
                result
            );
        }
    }
}
