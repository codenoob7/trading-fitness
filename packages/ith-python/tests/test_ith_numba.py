"""Tests for numba-accelerated ITH calculations."""

import numpy as np
import pandas as pd
import pytest

from ith_python.ith_numba import (
    ExcessGainLossResult,
    _excess_gain_excess_loss_numba_original,
    generate_synthetic_nav,
)


class TestExcessGainLossResult:
    """Tests for ExcessGainLossResult NamedTuple."""

    def test_result_has_expected_fields(self):
        """ExcessGainLossResult should have all expected fields."""
        result = ExcessGainLossResult(
            excess_gains=np.array([0.0, 0.1]),
            excess_losses=np.array([0.0, 0.05]),
            num_of_ith_epochs=1,
            ith_epochs=np.array([False, True]),
            ith_intervals_cv=0.5,
        )

        assert hasattr(result, "excess_gains")
        assert hasattr(result, "excess_losses")
        assert hasattr(result, "num_of_ith_epochs")
        assert hasattr(result, "ith_epochs")
        assert hasattr(result, "ith_intervals_cv")


class TestExcessGainExcessLoss:
    """Tests for the core excess gain/loss calculation."""

    def test_basic_calculation(self, sample_nav_array: np.ndarray):
        """Basic test that the function runs and returns expected types."""
        hurdle = 0.05
        result = _excess_gain_excess_loss_numba_original(sample_nav_array, hurdle)

        assert isinstance(result, ExcessGainLossResult)
        assert len(result.excess_gains) == len(sample_nav_array)
        assert len(result.excess_losses) == len(sample_nav_array)
        assert len(result.ith_epochs) == len(sample_nav_array)
        assert isinstance(result.num_of_ith_epochs, (int, np.integer))

    def test_flat_nav_no_epochs(self):
        """Flat NAV should produce zero ITH epochs."""
        flat_nav = np.ones(100)
        hurdle = 0.05
        result = _excess_gain_excess_loss_numba_original(flat_nav, hurdle)

        assert result.num_of_ith_epochs == 0

    def test_strong_uptrend_produces_epochs(self):
        """Strong uptrend should produce ITH epochs."""
        # Create strong upward trend
        nav = np.cumprod(1 + np.ones(100) * 0.02)  # 2% daily gains
        hurdle = 0.05
        result = _excess_gain_excess_loss_numba_original(nav, hurdle)

        assert result.num_of_ith_epochs > 0

    def test_excess_gains_non_negative(self, sample_nav_array: np.ndarray):
        """Excess gains should be non-negative."""
        hurdle = 0.05
        result = _excess_gain_excess_loss_numba_original(sample_nav_array, hurdle)

        assert np.all(result.excess_gains >= 0)

    def test_different_hurdles(self, sample_nav_array: np.ndarray):
        """Higher hurdle should produce fewer or equal ITH epochs."""
        result_low = _excess_gain_excess_loss_numba_original(sample_nav_array, 0.01)
        result_high = _excess_gain_excess_loss_numba_original(sample_nav_array, 0.10)

        assert result_high.num_of_ith_epochs <= result_low.num_of_ith_epochs


class TestGenerateSyntheticNav:
    """Tests for synthetic NAV generation."""

    def test_generates_dataframe(self):
        """Should return a pandas DataFrame."""
        nav = generate_synthetic_nav()
        assert isinstance(nav, pd.DataFrame)

    def test_has_nav_column(self):
        """Generated data should have NAV column."""
        nav = generate_synthetic_nav()
        assert "NAV" in nav.columns

    def test_has_date_index(self):
        """Generated data should have Date as index."""
        nav = generate_synthetic_nav()
        assert nav.index.name == "Date"
        assert isinstance(nav.index, pd.DatetimeIndex)

    def test_starts_at_one(self):
        """NAV should start at approximately 1.0."""
        nav = generate_synthetic_nav()
        assert abs(nav["NAV"].iloc[0] - 1.0) < 0.01

    def test_custom_date_range(self):
        """Should respect custom date range."""
        nav = generate_synthetic_nav(
            start_date="2021-01-01",
            end_date="2021-12-31",
        )
        assert nav.index[0] == pd.Timestamp("2021-01-01")
        assert nav.index[-1] == pd.Timestamp("2021-12-31")

    def test_nav_values_positive(self):
        """NAV values should be positive."""
        nav = generate_synthetic_nav()
        assert (nav["NAV"] > 0).all()
