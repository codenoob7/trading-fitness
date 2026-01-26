"""ClaSPy feature extraction for time series segmentation.

Extracts regime change features from NAV series using the ClaSP algorithm.
These features complement ITH epoch analysis by detecting structural breaks.

Reference: docs/features/CLASPY.md
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

import numpy as np

if TYPE_CHECKING:
    from numpy.typing import NDArray


@dataclass
class ClaspyFeatures:
    """Container for all ClaSPy-derived features."""

    # Primary features
    n_changepoints: int
    n_segments: int
    window_size: int

    # Profile statistics
    profile_mean: float
    profile_max: float
    profile_std: float

    # Segment statistics
    segment_mean_len: float
    segment_cv: float
    cp_density: float

    # Position features
    first_cp_idx: int
    last_cp_idx: int
    max_score_idx: int

    def to_dict(self, prefix: str = "clasp_") -> dict[str, float | int]:
        """Convert to dict with optional prefix for feature names."""
        return {
            f"{prefix}n_changepoints": self.n_changepoints,
            f"{prefix}n_segments": self.n_segments,
            f"{prefix}window_size": self.window_size,
            f"{prefix}profile_mean": self.profile_mean,
            f"{prefix}profile_max": self.profile_max,
            f"{prefix}profile_std": self.profile_std,
            f"{prefix}segment_mean_len": self.segment_mean_len,
            f"{prefix}segment_cv": self.segment_cv,
            f"{prefix}cp_density": self.cp_density,
            f"{prefix}first_cp_idx": self.first_cp_idx,
            f"{prefix}last_cp_idx": self.last_cp_idx,
            f"{prefix}max_score_idx": self.max_score_idx,
        }


def extract_claspy_features(
    nav: NDArray[np.floating],
    *,
    window_size: str | int = "suss",
    validation_method: str = "significance_test",
) -> ClaspyFeatures:
    """Extract all ClaSPy features from NAV series.

    Args:
        nav: NAV series as numpy array (normalized price starting at 1.0)
        window_size: Window size detection method or fixed integer.
            Options: "suss" (default), "fft", "acf", or integer
        validation_method: Change point validation method.
            Options: "significance_test" (default), "score_threshold"

    Returns:
        ClaspyFeatures dataclass with all extracted features

    Raises:
        ValueError: If NAV series is too short for segmentation

    Example:
        >>> import numpy as np
        >>> nav = np.array([1.0, 1.02, 1.05, 1.03, 1.08, 1.06, 1.10])
        >>> features = extract_claspy_features(nav)
        >>> features.n_segments
        1
    """
    # Lazy import to avoid loading claspy unless needed
    from claspy.segmentation import BinaryClaSPSegmentation

    min_length = 100  # ClaSPy minimum: 2 * excl_radius * window_size
    if len(nav) < min_length:
        msg = f"NAV series too short for ClaSPy: {len(nav)} < {min_length}"
        raise ValueError(msg)

    # Configure and run segmentation
    model = BinaryClaSPSegmentation(
        window_size=window_size,
        validation=validation_method,
    )
    change_points = model.fit_predict(nav)
    profile = model.profile

    # Calculate segment lengths
    if len(change_points) == 0:
        segment_lengths = np.array([len(nav)])
    else:
        boundaries = np.concatenate([[0], change_points, [len(nav)]])
        segment_lengths = np.diff(boundaries)

    # Calculate segment CV (coefficient of variation)
    segment_mean = float(np.mean(segment_lengths))
    if segment_mean > 0:
        segment_cv = float(np.std(segment_lengths) / segment_mean)
    else:
        segment_cv = np.nan

    return ClaspyFeatures(
        # Primary
        n_changepoints=len(change_points),
        n_segments=len(change_points) + 1,
        window_size=int(model.window_size),
        # Profile statistics
        profile_mean=float(np.nanmean(profile)),
        profile_max=float(np.nanmax(profile)),
        profile_std=float(np.nanstd(profile)),
        # Segment statistics
        segment_mean_len=segment_mean,
        segment_cv=segment_cv,
        cp_density=len(change_points) / len(nav),
        # Position features
        first_cp_idx=int(change_points[0]) if len(change_points) > 0 else -1,
        last_cp_idx=int(change_points[-1]) if len(change_points) > 0 else -1,
        max_score_idx=int(np.argmax(profile)),
    )


def extract_claspy_features_safe(
    nav: NDArray[np.floating],
    **kwargs,
) -> ClaspyFeatures | None:
    """Extract ClaSPy features with error handling.

    Same as extract_claspy_features but returns None on failure
    instead of raising exceptions. Useful for batch processing.

    Args:
        nav: NAV series as numpy array
        **kwargs: Passed to extract_claspy_features

    Returns:
        ClaspyFeatures or None if extraction failed
    """
    try:
        return extract_claspy_features(nav, **kwargs)
    except (ValueError, RuntimeError):
        return None
