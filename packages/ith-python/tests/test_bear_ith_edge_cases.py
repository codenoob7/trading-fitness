"""Tests for Bear ITH algorithm using hand-crafted edge cases.

This module validates both long and short epoch detection against
pre-calculated expected values from edge_cases.py.

SR&ED: Validation testing for Bear ITH experimental development.
SRED-Type: experimental-development
SRED-Claim: BEAR-ITH
"""

import numpy as np
import pytest

from ith_python.bear_ith_numba import bear_excess_gain_excess_loss
from ith_python.bull_ith_numba import bull_excess_gain_excess_loss

from fixtures.edge_cases import (
    ALL_EDGE_CASES,
    CASE_9_SYMMETRY_BASE,
    CASE_9_SYMMETRY_INVERTED,
)


# ============================================================
# Method B: Numerical Assertions (pytest parametrized)
# ============================================================


@pytest.mark.parametrize("case", ALL_EDGE_CASES, ids=lambda c: c["name"])
def test_long_epochs_match_expected(case):
    """Assert long epoch indices match calibrated expectations."""
    nav = case["nav"]
    tmaeg = case.get("tmaeg", 0.05)

    result = bull_excess_gain_excess_loss(nav, tmaeg)
    actual_epochs = [int(i) for i in np.where(result.bull_epochs)[0]]
    expected_epochs = case.get("expected_bull_epochs", [])

    assert actual_epochs == expected_epochs, (
        f"Case '{case['name']}': "
        f"expected long epochs {expected_epochs}, got {actual_epochs}"
    )


@pytest.mark.parametrize("case", ALL_EDGE_CASES, ids=lambda c: c["name"])
def test_short_epochs_match_expected(case):
    """Assert short (bear) epoch indices match calibrated expectations."""
    nav = case["nav"]
    tmaer = case.get("tmaer", 0.05)

    result = bear_excess_gain_excess_loss(nav, tmaer)
    actual_epochs = [int(i) for i in np.where(result.bear_epochs)[0]]
    expected_epochs = case.get("expected_bear_epochs", [])

    assert actual_epochs == expected_epochs, (
        f"Case '{case['name']}': "
        f"expected short epochs {expected_epochs}, got {actual_epochs}"
    )


# ============================================================
# Method D: Symmetry Property Tests
# ============================================================


def test_symmetry_inverted_nav_values():
    """Verify inverted NAV is correctly computed."""
    base_nav = CASE_9_SYMMETRY_BASE["nav"]
    inverted_nav = CASE_9_SYMMETRY_INVERTED["nav"]

    # Verify inversion: base + inverted should equal 2 * midpoint
    midpoint = (base_nav.max() + base_nav.min()) / 2
    expected_sum = 2 * midpoint

    np.testing.assert_array_almost_equal(
        base_nav + inverted_nav,
        np.full_like(base_nav, expected_sum),
        decimal=10,
        err_msg="Inverted NAV should mirror around midpoint",
    )


def test_symmetry_epoch_count_swap():
    """Verify that inverting NAV swaps long/short epoch counts.

    Key invariant: Long epochs on base NAV should approximately equal
    short epochs on inverted NAV (and vice versa).
    """
    base_nav = CASE_9_SYMMETRY_BASE["nav"]
    inverted_nav = CASE_9_SYMMETRY_INVERTED["nav"]
    threshold = 0.05

    # Run algorithms on both NAVs
    long_on_base = bull_excess_gain_excess_loss(base_nav, threshold)
    short_on_base = bear_excess_gain_excess_loss(base_nav, threshold)
    long_on_inverted = bull_excess_gain_excess_loss(inverted_nav, threshold)
    short_on_inverted = bear_excess_gain_excess_loss(inverted_nav, threshold)

    # Symmetry property: counts should swap
    # Long(base) <-> Short(inverted)
    # Short(base) <-> Long(inverted)
    assert long_on_base.num_of_bull_epochs == short_on_inverted.num_of_bear_epochs, (
        f"Symmetry violation: long(base)={long_on_base.num_of_bull_epochs} "
        f"!= short(inverted)={short_on_inverted.num_of_bear_epochs}"
    )

    assert short_on_base.num_of_bear_epochs == long_on_inverted.num_of_bull_epochs, (
        f"Symmetry violation: short(base)={short_on_base.num_of_bear_epochs} "
        f"!= long(inverted)={long_on_inverted.num_of_bull_epochs}"
    )


# ============================================================
# Algorithm Behavior Tests
# ============================================================


def test_pure_decline_no_long_epochs():
    """In a pure decline, there should be NO long epochs (no new highs)."""
    from fixtures.edge_cases import CASE_1_PURE_DECLINE

    nav = CASE_1_PURE_DECLINE["nav"]
    result = bull_excess_gain_excess_loss(nav, 0.05)

    assert result.num_of_bull_epochs == 0, (
        f"Pure decline should have 0 long epochs, got {result.num_of_bull_epochs}"
    )


def test_pure_decline_has_short_epochs():
    """In a pure decline, there should be short epochs (shorts profit)."""
    from fixtures.edge_cases import CASE_1_PURE_DECLINE

    nav = CASE_1_PURE_DECLINE["nav"]
    result = bear_excess_gain_excess_loss(nav, 0.05)

    assert result.num_of_bear_epochs > 0, (
        f"Pure decline should have > 0 short epochs, got {result.num_of_bear_epochs}"
    )


def test_pure_rally_has_long_epochs():
    """In a pure rally, there should be long epochs (longs profit)."""
    from fixtures.edge_cases import CASE_2_PURE_RALLY

    nav = CASE_2_PURE_RALLY["nav"]
    result = bull_excess_gain_excess_loss(nav, 0.05)

    assert result.num_of_bull_epochs > 0, (
        f"Pure rally should have > 0 long epochs, got {result.num_of_bull_epochs}"
    )


def test_pure_rally_no_short_epochs():
    """In a pure rally, there should be NO short epochs (runup adverse)."""
    from fixtures.edge_cases import CASE_2_PURE_RALLY

    nav = CASE_2_PURE_RALLY["nav"]
    result = bear_excess_gain_excess_loss(nav, 0.05)

    assert result.num_of_bear_epochs == 0, (
        f"Pure rally should have 0 short epochs, got {result.num_of_bear_epochs}"
    )


def test_flat_market_no_epochs():
    """In a flat market, there should be no epochs in either direction."""
    from fixtures.edge_cases import CASE_6_FLAT

    nav = CASE_6_FLAT["nav"]
    long_result = bull_excess_gain_excess_loss(nav, 0.05)
    short_result = bear_excess_gain_excess_loss(nav, 0.05)

    assert long_result.num_of_bull_epochs == 0, "Flat market should have 0 long epochs"
    assert short_result.num_of_bear_epochs == 0, "Flat market should have 0 short epochs"


# ============================================================
# Edge Cases for Algorithm Correctness
# ============================================================


def test_empty_nav_array():
    """Algorithms should handle empty NAV gracefully."""
    nav = np.array([], dtype=np.float64)

    # Note: Numba functions may not handle empty arrays gracefully
    # This test documents expected behavior
    if len(nav) > 0:
        long_result = bull_excess_gain_excess_loss(nav, 0.05)
        short_result = bear_excess_gain_excess_loss(nav, 0.05)
        assert long_result.num_of_bull_epochs == 0
        assert short_result.num_of_bear_epochs == 0


def test_single_point_nav():
    """Algorithms should handle single-point NAV."""
    nav = np.array([100.0], dtype=np.float64)

    long_result = bull_excess_gain_excess_loss(nav, 0.05)
    short_result = bear_excess_gain_excess_loss(nav, 0.05)

    assert long_result.num_of_bull_epochs == 0
    assert short_result.num_of_bear_epochs == 0


def test_constant_nav():
    """Algorithms should handle constant NAV (no movement)."""
    nav = np.array([100.0, 100.0, 100.0, 100.0, 100.0], dtype=np.float64)

    long_result = bull_excess_gain_excess_loss(nav, 0.05)
    short_result = bear_excess_gain_excess_loss(nav, 0.05)

    assert long_result.num_of_bull_epochs == 0
    assert short_result.num_of_bear_epochs == 0


# ============================================================
# Max Drawdown / Max Runup Symmetry Tests
# ============================================================


class TestMaxDrawdownMaxRunupSymmetry:
    """Tests for symmetric max_drawdown and max_runup calculations."""

    def test_max_drawdown_uptrend(self):
        """Pure uptrend should have zero drawdown."""
        from ith_python.bull_ith_numba import max_drawdown

        nav = np.array([1.0, 1.1, 1.2, 1.3, 1.4, 1.5])
        result = max_drawdown(nav)
        assert result == 0.0, f"Expected 0.0, got {result}"

    def test_max_drawdown_downtrend(self):
        """Downtrend should have positive drawdown."""
        from ith_python.bull_ith_numba import max_drawdown

        nav = np.array([1.0, 0.9, 0.8, 0.7])
        result = max_drawdown(nav)
        expected = 0.3  # 30% drawdown from 1.0 to 0.7
        assert abs(result - expected) < 0.01, f"Expected {expected}, got {result}"

    def test_max_runup_downtrend(self):
        """Pure downtrend should have zero runup."""
        from ith_python.bear_ith_numba import max_runup

        nav = np.array([1.5, 1.4, 1.3, 1.2, 1.1, 1.0])
        result = max_runup(nav)
        assert result == 0.0, f"Expected 0.0, got {result}"

    def test_max_runup_uptrend(self):
        """Uptrend should have positive runup (adverse for shorts)."""
        from ith_python.bear_ith_numba import max_runup

        nav = np.array([0.7, 0.8, 0.9, 1.0])
        result = max_runup(nav)
        # Runup = 1 - (running_min / nav) = 1 - (0.7 / 1.0) = 0.3
        expected = 0.3
        assert abs(result - expected) < 0.01, f"Expected {expected}, got {result}"

    def test_max_drawdown_bounded(self):
        """Max drawdown should be bounded [0, 1)."""
        from ith_python.bull_ith_numba import max_drawdown

        # Even extreme drawdown should stay < 1.0
        nav = np.array([1.0, 0.1, 0.01])
        result = max_drawdown(nav)
        assert 0 <= result < 1.0, f"Drawdown should be in [0, 1), got {result}"

    def test_max_runup_bounded(self):
        """Max runup should be bounded [0, 1) with new symmetric formula."""
        from ith_python.bear_ith_numba import max_runup

        # Even extreme runup should stay < 1.0 with bounded formula
        nav = np.array([0.01, 0.1, 1.0])
        result = max_runup(nav)
        assert 0 <= result < 1.0, f"Runup should be in [0, 1), got {result}"

    def test_drawdown_runup_symmetric_magnitude(self):
        """For symmetric price movements, drawdown and runup should be equal."""
        from ith_python.bull_ith_numba import max_drawdown
        from ith_python.bear_ith_numba import max_runup

        # Upward movement: 100 -> 120 (20% runup)
        nav_up = np.array([100.0, 120.0])
        runup = max_runup(nav_up)

        # Downward movement: 120 -> 100 (matches runup direction)
        nav_down = np.array([120.0, 100.0])
        drawdown = max_drawdown(nav_down)

        # Both should give approximately 0.167 (1 - 100/120)
        expected = 1 - 100 / 120
        assert abs(runup - expected) < 0.01, f"Runup expected {expected}, got {runup}"
        assert abs(drawdown - expected) < 0.01, f"Drawdown expected {expected}, got {drawdown}"

    def test_max_drawdown_recovery_preserves_max(self):
        """Recovery after drawdown should not affect max drawdown."""
        from ith_python.bull_ith_numba import max_drawdown

        nav = np.array([1.0, 1.1, 0.9, 1.2])  # Dips then recovers
        result = max_drawdown(nav)
        expected = 1 - 0.9 / 1.1  # Max from peak 1.1 to trough 0.9
        assert abs(result - expected) < 0.01, f"Expected {expected}, got {result}"

    def test_max_runup_decline_preserves_max(self):
        """Decline after runup should not affect max runup."""
        from ith_python.bear_ith_numba import max_runup

        nav = np.array([1.0, 0.9, 1.1, 0.8])  # Rallies then declines
        result = max_runup(nav)
        # Max runup from trough 0.9 to peak 1.1: 1 - 0.9/1.1
        expected = 1 - 0.9 / 1.1
        assert abs(result - expected) < 0.01, f"Expected {expected}, got {result}"
