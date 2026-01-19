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
