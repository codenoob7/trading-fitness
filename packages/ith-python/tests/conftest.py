"""Shared test fixtures for ith-python tests."""

import sys
from pathlib import Path

# Add tests directory to path for importing fixtures
tests_dir = Path(__file__).parent
if str(tests_dir) not in sys.path:
    sys.path.insert(0, str(tests_dir))

import numpy as np
import pandas as pd
import pytest
import tempfile
import shutil


@pytest.fixture
def sample_nav_data() -> pd.DataFrame:
    """Create sample NAV data for testing."""
    dates = pd.date_range("2020-01-01", periods=100, freq="D")
    # Simple upward trend with some noise
    nav_values = 1.0 + np.cumsum(np.random.randn(100) * 0.01 + 0.001)
    nav = pd.DataFrame({"NAV": nav_values}, index=dates)
    nav.index.name = "Date"
    return nav


@pytest.fixture
def sample_nav_with_pnl(sample_nav_data: pd.DataFrame) -> pd.DataFrame:
    """Create sample NAV data with PnL column."""
    nav = sample_nav_data.copy()
    nav["PnL"] = nav["NAV"].diff()
    nav["PnL"] = nav["PnL"].fillna(nav["NAV"].iloc[0] - 1)
    return nav


@pytest.fixture
def sample_nav_array() -> np.ndarray:
    """Create sample NAV array for numba functions."""
    np.random.seed(42)
    returns = np.random.randn(100) * 0.01 + 0.001
    nav = np.cumprod(1 + returns)
    return nav


@pytest.fixture
def temp_dir():
    """Create a temporary directory for test outputs."""
    tmp = tempfile.mkdtemp()
    yield Path(tmp)
    shutil.rmtree(tmp)


@pytest.fixture
def sample_csv_file(temp_dir: Path, sample_nav_with_pnl: pd.DataFrame) -> Path:
    """Create a sample CSV file for testing."""
    csv_path = temp_dir / "test_nav.csv"
    sample_nav_with_pnl.to_csv(csv_path)
    return csv_path


@pytest.fixture
def invalid_csv_file(temp_dir: Path) -> Path:
    """Create an invalid CSV file (missing NAV column)."""
    csv_path = temp_dir / "invalid.csv"
    df = pd.DataFrame({"Date": ["2020-01-01"], "Value": [1.0]})
    df.to_csv(csv_path, index=False)
    return csv_path


@pytest.fixture
def empty_csv_file(temp_dir: Path) -> Path:
    """Create an empty CSV file."""
    csv_path = temp_dir / "empty.csv"
    csv_path.touch()
    return csv_path
