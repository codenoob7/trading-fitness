// PROCESS-STORM-OK
//! ITH (Investment Time Horizon) calculations.

use crate::types::ExcessGainLossResult;

/// Calculate excess gains and losses for ITH epoch detection.
pub fn excess_gain_excess_loss(nav: &[f64], hurdle: f64) -> ExcessGainLossResult {
    let n = nav.len();
    if n == 0 {
        return ExcessGainLossResult {
            excess_gains: vec![],
            excess_losses: vec![],
            num_of_ith_epochs: 0,
            ith_epochs: vec![],
            ith_intervals_cv: f64::NAN,
        };
    }

    let mut excess_gains = vec![0.0; n];
    let mut excess_losses = vec![0.0; n];
    let mut ith_epochs = vec![false; n];

    let mut excess_gain = 0.0;
    let mut excess_loss = 0.0;
    let mut endorsing_crest = nav[0];
    let mut endorsing_nadir = nav[0];
    let mut candidate_crest = nav[0];
    let mut candidate_nadir = nav[0];

    for i in 1..n {
        let equity = nav[i - 1];
        let next_equity = nav[i];

        if next_equity > candidate_crest {
            excess_gain = if endorsing_crest != 0.0 {
                next_equity / endorsing_crest - 1.0
            } else {
                0.0
            };
            candidate_crest = next_equity;
        }

        if next_equity < candidate_nadir {
            excess_loss = 1.0 - next_equity / endorsing_crest;
            candidate_nadir = next_equity;
        }

        let reset_condition = excess_gain > excess_loss.abs()
            && excess_gain > hurdle
            && candidate_crest >= endorsing_crest;

        if reset_condition {
            endorsing_crest = candidate_crest;
            endorsing_nadir = equity;
            candidate_nadir = equity;
        } else {
            endorsing_nadir = endorsing_nadir.min(equity);
        }

        excess_gains[i] = excess_gain;
        excess_losses[i] = excess_loss;

        if reset_condition {
            excess_gain = 0.0;
            excess_loss = 0.0;
        }

        let ith_epoch_condition = excess_gains[i] > excess_losses[i] && excess_gains[i] > hurdle;
        ith_epochs[i] = ith_epoch_condition;
    }

    let num_of_ith_epochs = ith_epochs.iter().filter(|&&x| x).count();
    let ith_intervals_cv = calculate_ith_intervals_cv(&ith_epochs);

    ExcessGainLossResult {
        excess_gains,
        excess_losses,
        num_of_ith_epochs,
        ith_epochs,
        ith_intervals_cv,
    }
}

fn calculate_ith_intervals_cv(ith_epochs: &[bool]) -> f64 {
    let mut epoch_indices: Vec<usize> = vec![0];
    for (i, &is_epoch) in ith_epochs.iter().enumerate() {
        if is_epoch {
            epoch_indices.push(i);
        }
    }

    if epoch_indices.len() < 2 {
        return f64::NAN;
    }

    let intervals: Vec<f64> = epoch_indices
        .windows(2)
        .map(|w| (w[1] - w[0]) as f64)
        .collect();

    if intervals.is_empty() {
        return f64::NAN;
    }

    let n = intervals.len() as f64;
    let mean = intervals.iter().sum::<f64>() / n;

    if mean == 0.0 {
        return f64::NAN;
    }

    let variance = intervals.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
    variance.sqrt() / mean
}

/// Determine TMAEG from NAV data.
pub fn determine_tmaeg(nav: &[f64], method: &str, fixed_value: f64) -> f64 {
    match method {
        "mdd" => crate::metrics::max_drawdown(nav),
        "fixed" => fixed_value,
        _ => fixed_value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_nav_no_epochs() {
        let nav: Vec<f64> = vec![1.0; 100];
        let result = excess_gain_excess_loss(&nav, 0.05);
        assert_eq!(result.num_of_ith_epochs, 0);
    }

    #[test]
    fn test_uptrend_produces_epochs() {
        let nav: Vec<f64> = (0..100).map(|i| 1.02_f64.powi(i)).collect();
        let result = excess_gain_excess_loss(&nav, 0.05);
        assert!(result.num_of_ith_epochs > 0);
    }

    #[test]
    fn test_excess_gains_non_negative() {
        let nav: Vec<f64> = (0..100).map(|i| 1.0 + 0.01 * i as f64).collect();
        let result = excess_gain_excess_loss(&nav, 0.05);
        assert!(result.excess_gains.iter().all(|&g| g >= 0.0));
    }

    #[test]
    fn test_different_hurdles() {
        let nav: Vec<f64> = (0..100).map(|i| 1.02_f64.powi(i)).collect();
        let result_low = excess_gain_excess_loss(&nav, 0.01);
        let result_high = excess_gain_excess_loss(&nav, 0.10);
        assert!(result_high.num_of_ith_epochs <= result_low.num_of_ith_epochs);
    }

    #[test]
    fn test_result_lengths_match_input() {
        let nav: Vec<f64> = vec![1.0, 1.1, 1.2, 1.15, 1.25];
        let result = excess_gain_excess_loss(&nav, 0.05);
        assert_eq!(result.excess_gains.len(), nav.len());
    }

    #[test]
    fn test_determine_tmaeg_mdd() {
        let nav = vec![1.0, 1.1, 0.9, 1.0];
        let tmaeg = determine_tmaeg(&nav, "mdd", 0.05);
        assert!(tmaeg > 0.15);
    }

    #[test]
    fn test_determine_tmaeg_fixed() {
        let nav = vec![1.0, 1.1, 0.9, 1.0];
        let tmaeg = determine_tmaeg(&nav, "fixed", 0.05);
        assert_eq!(tmaeg, 0.05);
    }
}
