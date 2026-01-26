# ClaSPy Integration

> Time series segmentation for regime detection in trading fitness analysis.

**← [Back to Feature Registry](./REGISTRY.md)** | **← [Back to CLAUDE.md](../../CLAUDE.md)**

## Overview

ClaSPy provides unsupervised time series segmentation using the ClaSP algorithm. We use it to detect **regime changes** in NAV series, complementing ITH epoch analysis.

**Fork Location**: `~/fork-tools/claspy/`
**Upstream**: `github.com/ermshaua/claspy`

---

## Installation

```bash
# Already available via fork
pip install claspy

# Or from fork
pip install -e ~/fork-tools/claspy/
```

---

## Quick Usage

```python
from claspy.segmentation import BinaryClaSPSegmentation
import numpy as np

# NAV series (normalized price)
nav = np.array([1.0, 1.02, 1.05, ...])

# Detect regime changes
model = BinaryClaSPSegmentation()
change_points = model.fit_predict(nav)

# Access score profile (change probability)
profile = model.profile  # Array of [0, 1] scores
```

---

## Extractable Features

### Primary Features

| Feature                | Extraction                      | Description                  |
| ---------------------- | ------------------------------- | ---------------------------- |
| `clasp_n_changepoints` | `len(model.fit_predict(nav))`   | Number of detected changes   |
| `clasp_n_segments`     | `len(change_points) + 1`        | Number of segments           |
| `clasp_window_size`    | `model.window_size` (after fit) | Auto-detected pattern length |

### Profile Statistics

| Feature              | Extraction               | Description                |
| -------------------- | ------------------------ | -------------------------- |
| `clasp_profile_mean` | `np.mean(model.profile)` | Average change probability |
| `clasp_profile_max`  | `np.max(model.profile)`  | Strongest change signal    |
| `clasp_profile_std`  | `np.std(model.profile)`  | Profile variability        |

### Segment Statistics

| Feature                  | Extraction                  | Description              |
| ------------------------ | --------------------------- | ------------------------ |
| `clasp_segment_mean_len` | Mean of segment lengths     | Average segment duration |
| `clasp_segment_cv`       | CV of segment lengths       | Segment regularity       |
| `clasp_cp_density`       | `n_changepoints / len(nav)` | Change frequency         |

### Position Features

| Feature               | Extraction                 | Description               |
| --------------------- | -------------------------- | ------------------------- |
| `clasp_first_cp_idx`  | `change_points[0]`         | First change location     |
| `clasp_last_cp_idx`   | `change_points[-1]`        | Last change location      |
| `clasp_max_score_idx` | `np.argmax(model.profile)` | Strongest change location |

---

## Configuration Options

### Distance Metrics

| Metric                          | Use Case                           |
| ------------------------------- | ---------------------------------- |
| `znormed_euclidean_distance`    | Default, handles scale differences |
| `euclidean_distance`            | Raw distance, sensitive to scale   |
| `cinvariant_euclidean_distance` | Complexity-adjusted                |

### Window Size Detection

| Method | Parameter            | Description                  |
| ------ | -------------------- | ---------------------------- |
| SUSS   | `window_size="suss"` | Summary statistics (default) |
| FFT    | `window_size="fft"`  | Dominant Fourier frequency   |
| ACF    | `window_size="acf"`  | Highest autocorrelation      |
| Fixed  | `window_size=50`     | Manual specification         |

### Validation Methods

| Method              | Threshold | Use Case             |
| ------------------- | --------- | -------------------- |
| `significance_test` | `1e-15`   | Statistical rigor    |
| `score_threshold`   | `0.75`    | Application-specific |

---

## Integration with ITH

### Feature Extractor

```python
def extract_claspy_features(nav: np.ndarray) -> dict:
    """Extract all ClaSPy features from NAV series."""
    from claspy.segmentation import BinaryClaSPSegmentation

    model = BinaryClaSPSegmentation()
    cps = model.fit_predict(nav)
    profile = model.profile

    # Handle edge cases
    if len(cps) == 0:
        segment_lengths = [len(nav)]
    else:
        segment_lengths = np.diff([0] + list(cps) + [len(nav)])

    return {
        "clasp_n_changepoints": len(cps),
        "clasp_n_segments": len(cps) + 1,
        "clasp_window_size": model.window_size,
        "clasp_profile_mean": float(np.nanmean(profile)),
        "clasp_profile_max": float(np.nanmax(profile)),
        "clasp_profile_std": float(np.nanstd(profile)),
        "clasp_segment_mean_len": float(np.mean(segment_lengths)),
        "clasp_segment_cv": float(np.std(segment_lengths) / np.mean(segment_lengths))
            if np.mean(segment_lengths) > 0 else np.nan,
        "clasp_cp_density": len(cps) / len(nav),
        "clasp_first_cp_idx": int(cps[0]) if len(cps) > 0 else -1,
        "clasp_last_cp_idx": int(cps[-1]) if len(cps) > 0 else -1,
        "clasp_max_score_idx": int(np.argmax(profile)),
    }
```

### Correlation with ITH Epochs

Research question: Do ClaSP change points correlate with ITH epoch boundaries?

```python
# Compare change points to epoch events
epoch_indices = np.where(bull_epochs)[0]
clasp_indices = model.fit_predict(nav)

# Measure alignment
from scipy.spatial.distance import cdist
distances = cdist(epoch_indices.reshape(-1,1), clasp_indices.reshape(-1,1))
min_distances = distances.min(axis=1)
alignment_score = np.mean(min_distances < window_size)
```

---

## Streaming Integration

For real-time trading applications:

```python
from claspy.streaming.segmentation import StreamingClaSPSegmentation

model = StreamingClaSPSegmentation(
    n_timepoints=5000,    # Buffer size
    n_warmup=1000,        # Calibration period
    jump=5,               # Detection frequency
)

for price in price_stream:
    model.update(price)
    if model.warmup_counter >= model.n_warmup:
        cp = model.predict()
        if cp > 0:
            # Regime change detected!
            emit_regime_change_event(cp)
```

---

## Performance Characteristics

| Metric            | Value        | Notes                      |
| ----------------- | ------------ | -------------------------- |
| Time complexity   | O(n² · d)    | n=timepoints, d=dimensions |
| Space complexity  | O(n · k)     | k=neighbors                |
| Batch throughput  | ~10k pts/sec | On typical hardware        |
| Streaming latency | <1ms/update  | After warmup               |

### Minimum Data Requirements

- Batch: ≥ 100 data points (2 × excl_radius × window_size)
- Streaming: ≥ n_warmup points before detection

---

## Telemetry Events

ClaSPy feature extraction emits telemetry events:

```json
{
  "ts": "2026-01-25T00:00:00Z",
  "event_type": "claspy.segmentation",
  "trace_id": "abc123",
  "n_changepoints": 5,
  "window_size": 42,
  "profile_max": 0.92,
  "duration_ms": 150
}
```

---

## Redundancy Analysis

Expected correlations with existing features:

| ClaSPy Feature     | Potentially Redundant With  | Expected Correlation |
| ------------------ | --------------------------- | -------------------- |
| `clasp_n_segments` | `bull_epochs + bear_epochs` | Medium               |
| `clasp_segment_cv` | `bull_cv`, `bear_cv`        | Medium               |
| `clasp_cp_density` | Epoch frequency             | Medium               |
| `clasp_profile_*`  | (Novel)                     | Low                  |

Run statistical examination to validate.

---

## Related Documentation

- [Feature Registry](./REGISTRY.md)
- [Statistical Examination](../../packages/ith-python/src/ith_python/statistical_examination/CLAUDE.md)
- [Telemetry Module](../../packages/ith-python/src/ith_python/telemetry/)
- [ClaSPy Upstream](https://github.com/ermshaua/claspy)
