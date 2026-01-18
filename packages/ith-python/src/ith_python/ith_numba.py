"""Numba-accelerated ITH calculations.

This module provides JIT-compiled functions for calculating excess gain/loss
and ITH (Investment Time Horizon) epochs efficiently.
"""

from __future__ import annotations

from typing import NamedTuple

import numpy as np
import pandas as pd
from numba import njit
from scipy import stats


class ExcessGainLossResult(NamedTuple):
    """Result of excess gain/loss calculation."""

    excess_gains: np.ndarray
    excess_losses: np.ndarray
    num_of_ith_epochs: int
    ith_epochs: np.ndarray
    ith_intervals_cv: float


@njit
def _excess_gain_excess_loss_numba_original(nav, hurdle):
    """Calculate excess gains/losses using pre-allocated numpy arrays.

    This version uses fixed-size numpy arrays instead of growing lists
    for compatibility with newer numba versions.
    """
    n = len(nav)

    # Pre-allocate arrays
    excess_gains = np.zeros(n, dtype=np.float64)
    excess_losses = np.zeros(n, dtype=np.float64)
    ith_epochs = np.zeros(n, dtype=np.bool_)

    excess_gain = 0.0
    excess_loss = 0.0
    endorsing_crest = nav[0]
    endorsing_nadir = nav[0]
    candidate_crest = nav[0]
    candidate_nadir = nav[0]

    for i in range(1, n):
        equity = nav[i - 1]
        next_equity = nav[i]

        if next_equity > candidate_crest:
            if endorsing_crest != 0:
                excess_gain = next_equity / endorsing_crest - 1
            else:
                excess_gain = 0.0
            candidate_crest = next_equity

        if next_equity < candidate_nadir:
            excess_loss = 1 - next_equity / endorsing_crest
            candidate_nadir = next_equity

        reset_condition = (
            excess_gain > abs(excess_loss)
            and excess_gain > hurdle
            and candidate_crest >= endorsing_crest
        )

        if reset_condition:
            endorsing_crest = candidate_crest
            endorsing_nadir = equity
            candidate_nadir = equity
        else:
            endorsing_nadir = min(endorsing_nadir, equity)

        excess_gains[i] = excess_gain
        excess_losses[i] = excess_loss

        if reset_condition:
            excess_gain = 0.0
            excess_loss = 0.0

        # Check ITH epoch condition
        ith_epoch_condition = (
            excess_gains[i] > excess_losses[i] and excess_gains[i] > hurdle
        )
        ith_epochs[i] = ith_epoch_condition

    # Count ITH epochs
    num_of_ith_epochs = 0
    for i in range(n):
        if ith_epochs[i]:
            num_of_ith_epochs += 1

    # Calculate ITH intervals CV
    # Find indices where ith_epochs is True
    epoch_indices = np.zeros(num_of_ith_epochs + 1, dtype=np.int64)
    epoch_indices[0] = 0
    idx = 1
    for i in range(n):
        if ith_epochs[i]:
            epoch_indices[idx] = i
            idx += 1

    if num_of_ith_epochs > 0:
        ith_intervals = np.diff(epoch_indices[: num_of_ith_epochs + 1])
        if len(ith_intervals) > 0:
            mean_interval = np.mean(ith_intervals)
            if mean_interval > 0:
                ith_intervals_cv = np.std(ith_intervals) / mean_interval
            else:
                ith_intervals_cv = np.nan
        else:
            ith_intervals_cv = np.nan
    else:
        ith_intervals_cv = np.nan

    return ExcessGainLossResult(
        excess_gains=excess_gains,
        excess_losses=excess_losses,
        num_of_ith_epochs=num_of_ith_epochs,
        ith_epochs=ith_epochs,
        ith_intervals_cv=ith_intervals_cv,
    )

def generate_synthetic_nav(
    start_date: str = "2020-01-01",
    end_date: str = "2023-01-01",
    avg_daily_return: float = 0.0001,
    daily_return_volatility: float = 0.01,
    df: int = 5,
) -> pd.DataFrame:
    """Generate synthetic NAV data using t-distribution returns.

    Args:
        start_date: Start date for the NAV series.
        end_date: End date for the NAV series.
        avg_daily_return: Average daily return.
        daily_return_volatility: Daily return volatility.
        df: Degrees of freedom for the t-distribution.

    Returns:
        DataFrame with Date index and NAV column.
    """
    dates = pd.date_range(start_date, end_date)
    walk = stats.t.rvs(
        df, loc=avg_daily_return, scale=daily_return_volatility, size=len(dates)
    )
    walk = np.cumsum(walk)
    walk = walk - walk[0] + 1  # Normalize the series so that it starts with 1
    nav = pd.DataFrame(data=walk, index=dates, columns=["NAV"])
    nav.index.name = "Date"
    return nav


if __name__ == "__main__":
    # Generate synthetic NAV data and test the function
    nav = generate_synthetic_nav()
    hurdle = 0.05

    result = _excess_gain_excess_loss_numba_original(nav["NAV"].values, hurdle)
    print(f"Generated {result.num_of_ith_epochs} ITH epochs")
    print(f"ITH intervals CV: {result.ith_intervals_cv:.4f}")