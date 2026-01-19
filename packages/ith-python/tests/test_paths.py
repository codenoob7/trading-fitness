"""Tests for path configuration utilities."""

from pathlib import Path

from ith_python.paths import (
    REPO_ROOT,
    get_data_dir,
    get_artifacts_dir,
    get_log_dir,
    get_custom_nav_dir,
    get_synth_bull_ithes_dir,
    get_synth_bear_ithes_dir,
    get_synth_ithes_dir,  # Deprecated alias for backwards compatibility
    ensure_dirs,
)


class TestRepoRoot:
    """Tests for REPO_ROOT constant."""

    def test_repo_root_exists(self):
        """REPO_ROOT should point to an existing directory."""
        assert REPO_ROOT.exists()
        assert REPO_ROOT.is_dir()

    def test_repo_root_has_expected_structure(self):
        """REPO_ROOT should contain expected monorepo directories."""
        assert (REPO_ROOT / "packages").exists()
        assert (REPO_ROOT / "CLAUDE.md").exists()


class TestPathFunctions:
    """Tests for path getter functions."""

    def test_get_data_dir(self):
        """get_data_dir returns path under REPO_ROOT."""
        data_dir = get_data_dir()
        assert data_dir == REPO_ROOT / "data"
        assert isinstance(data_dir, Path)

    def test_get_artifacts_dir(self):
        """get_artifacts_dir returns path under REPO_ROOT."""
        artifacts_dir = get_artifacts_dir()
        assert artifacts_dir == REPO_ROOT / "artifacts"
        assert isinstance(artifacts_dir, Path)

    def test_get_log_dir(self):
        """get_log_dir returns path under REPO_ROOT."""
        log_dir = get_log_dir()
        assert log_dir == REPO_ROOT / "logs"
        assert isinstance(log_dir, Path)

    def test_get_custom_nav_dir(self):
        """get_custom_nav_dir returns correct nested path."""
        custom_nav_dir = get_custom_nav_dir()
        assert custom_nav_dir == REPO_ROOT / "data" / "nav_data_custom"
        assert isinstance(custom_nav_dir, Path)

    def test_get_synth_bull_ithes_dir(self):
        """get_synth_bull_ithes_dir returns correct nested path."""
        synth_dir = get_synth_bull_ithes_dir()
        assert synth_dir == REPO_ROOT / "artifacts" / "synth_bull_ithes"
        assert isinstance(synth_dir, Path)

    def test_get_synth_bear_ithes_dir(self):
        """get_synth_bear_ithes_dir returns correct nested path."""
        synth_dir = get_synth_bear_ithes_dir()
        assert synth_dir == REPO_ROOT / "artifacts" / "synth_bear_ithes"
        assert isinstance(synth_dir, Path)

    def test_get_synth_ithes_dir_deprecated_alias(self):
        """get_synth_ithes_dir (deprecated) should match get_synth_bull_ithes_dir."""
        assert get_synth_ithes_dir() == get_synth_bull_ithes_dir()


class TestEnsureDirs:
    """Tests for ensure_dirs function."""

    def test_ensure_dirs_creates_all_directories(self):
        """ensure_dirs should create all required directories."""
        ensure_dirs()

        assert get_data_dir().exists()
        assert get_artifacts_dir().exists()
        assert get_log_dir().exists()
        assert get_custom_nav_dir().exists()
        assert get_synth_bull_ithes_dir().exists()
        assert get_synth_bear_ithes_dir().exists()

    def test_ensure_dirs_is_idempotent(self):
        """Calling ensure_dirs multiple times should not raise errors."""
        ensure_dirs()
        ensure_dirs()  # Should not raise
        ensure_dirs()  # Should not raise

        assert get_data_dir().exists()
