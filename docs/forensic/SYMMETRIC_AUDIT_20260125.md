# Symmetric Feature Selection Forensic Audit - 2026-01-25

> Principled feature selection preserving bull/bear symmetry for ML interpretability.

**[Back to E2E.md](./E2E.md)** | **[Back to CLAUDE.md](../../CLAUDE.md)** | **[Previous: COMPREHENSIVE_AUDIT](./COMPREHENSIVE_AUDIT_20260125.md)**

---

## Executive Summary

| Dimension                | Status | Key Finding                                        |
| ------------------------ | ------ | -------------------------------------------------- | --- | ------- |
| Selection Approach       | PASS   | Symmetric pairing preserves interpretability       |
| Threshold Redundancy     | FIXED  | 4 → 1 threshold (r=1.0 redundancy eliminated)      |
| Lookback Selection       | PASS   | 5 → 3 lookbacks (distinct multi-scale)             |
| Orthogonality Validation | PASS   | 88% of pairs orthogonal (                          | r   | < 0.30) |
| Final Feature Count      | 24     | 85% reduction from 160, all symmetric pairs intact |

**Key Principle**: Semantic correctness over statistical optimization. Always keep bull and bear versions together.

---

## 1. Problem Statement

### 1.1 Why Greedy Selection Failed

The previous analysis used greedy selection based purely on statistical orthogonality:

```
Greedy Selection Result:
- Selected: bull_cv (lb200), bear_eg (lb100), bull_eg (lb100), ...
- Problem: Mixed bull/bear features at different lookbacks
- Result: BROKEN SYMMETRY
```

**Issues with Greedy Selection**:

1. **Directional Bias**: Model might see more bull features than bear features
2. **Interpretability Loss**: Cannot compare bull vs bear behavior at same scale
3. **Overfitting Risk**: Optimized for this dataset's correlation structure
4. **Generalization Failure**: Data-specific selection may not transfer

### 1.2 Symmetric Selection Principle

**Rule**: Always keep BOTH bull and bear versions of each metric together.

```
┌─────────────────────────────────────────────────────────────────┐
│ If you keep bull_cv at lb100, you MUST also keep bear_cv at lb100 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. Symmetric Pair Definitions

| Pair Name | Bull Feature | Bear Feature | What It Measures                    |
| --------- | ------------ | ------------ | ----------------------------------- |
| Timing    | bull_cv      | bear_cv      | Regularity of epoch intervals       |
| Density   | bull_ed      | bear_ed      | Frequency of threshold crossings    |
| Excess    | bull_eg      | bear_eg      | Magnitude before threshold triggers |
| Risk      | max_dd       | max_ru       | Maximum adverse excursion           |

### 2.1 Intra-Pair Correlations

How correlated are bull and bear versions of the same metric?

| Pair    | Avg Correlation | Interpretation                              |
| ------- | --------------- | ------------------------------------------- |
| Timing  | -0.07           | Nearly independent (uptrend ≠ downtrend)    |
| Density | -0.22           | Weakly inverse (more bull → fewer bear)     |
| Excess  | -0.19           | Weakly inverse                              |
| Risk    | -0.45           | Moderately inverse (big runups → drawdowns) |

**Key Insight**: Bull and bear features provide DIFFERENT information. Keeping both is not redundant.

---

## 3. Threshold Analysis

### 3.1 Threshold Redundancy (Root Cause)

All thresholds produce identical feature values (r = 1.0):

| Comparison                     | Correlation |
| ------------------------------ | ----------- |
| rb25 vs rb50 (same lookback)   | 1.000       |
| rb50 vs rb100 (same lookback)  | 1.000       |
| rb100 vs rb250 (same lookback) | 1.000       |

**Root Cause**: TMAEG is auto-calculated from data volatility using MAD-based estimation. The `threshold_dbps` parameter is for column naming only, not computation.

### 3.2 Decision

**Action**: Keep only `threshold=100` (rb100)

**Result**: 160 → 40 features (75% reduction)

---

## 4. Lookback Analysis

### 4.1 Cross-Lookback Correlations

| Lookback Pair | Avg \|r\| | Classification   |
| ------------- | --------- | ---------------- |
| lb20 ↔ lb50   | 0.474     | Moderate overlap |
| lb20 ↔ lb100  | 0.309     | DISTINCT         |
| lb20 ↔ lb200  | 0.165     | DISTINCT         |
| lb20 ↔ lb500  | 0.096     | HIGHLY DISTINCT  |
| lb50 ↔ lb100  | 0.563     | Moderate overlap |
| lb50 ↔ lb200  | 0.275     | DISTINCT         |
| lb50 ↔ lb500  | 0.149     | DISTINCT         |
| lb100 ↔ lb200 | 0.507     | Moderate overlap |
| lb100 ↔ lb500 | 0.279     | DISTINCT         |
| lb200 ↔ lb500 | 0.523     | Moderate overlap |

### 4.2 Decision

**Keep**: lb20, lb100, lb500 (maximally distinct scales)

**Drop**: lb50, lb200 (high correlation with neighbors)

**Result**: 40 → 24 features (40% reduction)

---

## 5. Final Feature Set (24 Features)

### 5.1 Feature Matrix

```
              │ bull_cv │ bear_cv │ bull_ed │ bear_ed │ bull_eg │ bear_eg │ max_dd │ max_ru │
──────────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────┼────────┼────────┤
lb20 (short)  │    ✓    │    ✓    │    ✓    │    ✓    │    ✓    │    ✓    │   ✓    │   ✓    │
lb100 (medium)│    ✓    │    ✓    │    ✓    │    ✓    │    ✓    │    ✓    │   ✓    │   ✓    │
lb500 (long)  │    ✓    │    ✓    │    ✓    │    ✓    │    ✓    │    ✓    │   ✓    │   ✓    │
```

### 5.2 Complete Feature List

**lb20 (Short-term / Micro Patterns)**:

- `ith_rb100_lb20_bull_cv` ↔ `ith_rb100_lb20_bear_cv`
- `ith_rb100_lb20_bull_ed` ↔ `ith_rb100_lb20_bear_ed`
- `ith_rb100_lb20_bull_eg` ↔ `ith_rb100_lb20_bear_eg`
- `ith_rb100_lb20_max_dd` ↔ `ith_rb100_lb20_max_ru`

**lb100 (Medium-term / Regime Patterns)**:

- `ith_rb100_lb100_bull_cv` ↔ `ith_rb100_lb100_bear_cv`
- `ith_rb100_lb100_bull_ed` ↔ `ith_rb100_lb100_bear_ed`
- `ith_rb100_lb100_bull_eg` ↔ `ith_rb100_lb100_bear_eg`
- `ith_rb100_lb100_max_dd` ↔ `ith_rb100_lb100_max_ru`

**lb500 (Long-term / Macro Trends)**:

- `ith_rb100_lb500_bull_cv` ↔ `ith_rb100_lb500_bear_cv`
- `ith_rb100_lb500_bull_ed` ↔ `ith_rb100_lb500_bear_ed`
- `ith_rb100_lb500_bull_eg` ↔ `ith_rb100_lb500_bear_eg`
- `ith_rb100_lb500_max_dd` ↔ `ith_rb100_lb500_max_ru`

---

## 6. Orthogonality Validation

### 6.1 Correlation Distribution

| Correlation Range                 | Pairs | Percentage |
| --------------------------------- | ----- | ---------- |
| Highly correlated (≥0.90)         | 1     | 0.4%       |
| Moderately correlated (0.50-0.90) | 12    | 4.3%       |
| Low correlation (0.30-0.50)       | 20    | 7.2%       |
| Orthogonal (<0.30)                | 243   | 88.0%      |

**Statistics**:

- Mean |r|: 0.1209
- Median |r|: 0.0561
- Total pairs: 276

### 6.2 Cross-Scale Correlation Matrix (lb20 vs lb100)

```
           bull_cv bear_cv bull_ed bear_ed bull_eg bear_eg  max_dd  max_ru
bull_cv      +0.06   -0.07   +0.08   -0.06   -0.02   +0.03   -0.06   +0.17
bear_cv      -0.05   +0.07   -0.09   +0.06   -0.03   +0.02   +0.15   -0.04
bull_ed      +0.04   -0.07   +0.06   -0.06   -0.04   +0.06   -0.07   +0.19
bear_ed      -0.02   +0.23   -0.05   +0.06   +0.02   +0.05   +0.15   -0.04
bull_eg      +0.07   -0.14   +0.06   -0.12   -0.01   -0.03   -0.14   +0.06
bear_eg      -0.07   -0.03   -0.11   +0.01   -0.02   -0.00   +0.04   -0.14
max_dd       -0.08   +0.12   -0.16   +0.04   +0.03   -0.01   +0.22   -0.15
max_ru       +0.10   -0.24   +0.09   -0.17   -0.10   +0.06   -0.18   +0.17
```

**Interpretation**: Cross-scale correlations are very low (most < 0.20), confirming that different lookbacks provide distinct information.

---

## 7. Semantic Interpretation

### 7.1 Feature Pair Meanings

**TIMING PAIR (bull_cv ↔ bear_cv)**

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Measures: Regularity of profit-taking/loss-cutting intervals                │
│ Low CV:   Regular, predictable rhythm (steady accumulation/distribution)    │
│ High CV:  Erratic timing (clustered events, long quiet periods)             │
│ Bull/Bear correlation: -0.07 (nearly independent)                           │
│ → Uptrend rhythm doesn't predict downtrend rhythm                           │
└─────────────────────────────────────────────────────────────────────────────┘
```

**DENSITY PAIR (bull_ed ↔ bear_ed)**

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Measures: How frequently threshold crossings occur (epochs per bar)         │
│ Low ED:   Few threshold events (trending or low volatility)                 │
│ High ED:  Many threshold events (choppy, mean-reverting)                    │
│ Bull/Bear correlation: -0.22 (weakly inverse)                               │
│ → Assets with frequent upside triggers have fewer downside triggers         │
└─────────────────────────────────────────────────────────────────────────────┘
```

**EXCESS PAIR (bull_eg ↔ bear_eg)**

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Measures: Magnitude of unrealized gains/losses before threshold triggers    │
│ Low EG:   Small moves before trigger (tight thresholds effective)           │
│ High EG:  Large moves before trigger (significant excursions)               │
│ Bull/Bear correlation: -0.19 (weakly inverse)                               │
│ → Upside magnitude is independent of downside magnitude                     │
└─────────────────────────────────────────────────────────────────────────────┘
```

**RISK PAIR (max_dd ↔ max_ru)**

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Measures: Maximum adverse excursion in each direction                       │
│ Low DD/RU: Shallow pullbacks/rallies (range-bound)                          │
│ High DD/RU: Deep drawdowns/runups (trending/volatile)                       │
│ Bull/Bear correlation: -0.45 (moderately inverse)                           │
│ → Strongest inverse relationship: deep drawdowns often follow big runups    │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 7.2 Lookback Scale Meanings

| Lookback | Scale       | Captures                   | Use Case                 |
| -------- | ----------- | -------------------------- | ------------------------ |
| lb20     | Short-term  | Micro patterns, noise      | HF signals, noise filter |
| lb100    | Medium-term | Regime transitions         | Trend following          |
| lb500    | Long-term   | Major cycles, macro trends | Strategic positioning    |

---

## 8. Comparison: Greedy vs Symmetric

| Criterion              | Greedy (Previous) | Symmetric (New) |
| ---------------------- | ----------------- | --------------- |
| Features               | 20                | 24              |
| Reduction              | 87.5%             | 85%             |
| Bull/Bear Symmetry     | **BROKEN**        | **PRESERVED**   |
| Interpretability       | Low               | High            |
| Directional Bias Risk  | High              | None            |
| Generalization         | Overfit risk      | Principled      |
| Multi-scale Coverage   | Uneven            | Balanced        |
| Cross-scale Comparison | Inconsistent      | Consistent      |

**Trade-off**: 4 additional features (24 vs 20) is acceptable for interpretability and generalization benefits.

---

## 9. Implementation

### 9.1 Feature Filter Code

```python
# Symmetric feature selection
SELECTED_THRESHOLD = 100
SELECTED_LOOKBACKS = [20, 100, 500]
FEATURE_PAIRS = [
    ("bull_cv", "bear_cv"),
    ("bull_ed", "bear_ed"),
    ("bull_eg", "bear_eg"),
    ("max_dd", "max_ru"),
]

def get_symmetric_features():
    features = []
    for lb in SELECTED_LOOKBACKS:
        for bull, bear in FEATURE_PAIRS:
            features.append(f"ith_rb{SELECTED_THRESHOLD}_lb{lb}_{bull}")
            features.append(f"ith_rb{SELECTED_THRESHOLD}_lb{lb}_{bear}")
    return features  # 24 features
```

### 9.2 Polars Filter

```python
import polars as pl

df = pl.read_parquet("artifacts/ssot/features_long.parquet")

# Filter to symmetric set
symmetric_df = df.filter(
    (pl.col("threshold_dbps") == 100) &
    (pl.col("lookback").is_in([20, 100, 500])) &
    (pl.col("valid") == True)
)
```

---

## 10. Recommendations

### 10.1 For ML Modeling

1. **Use 24-feature symmetric set** for all ITH-based models
2. **TimeSeriesSplit CV** required (high autocorrelation)
3. **Compare bull vs bear coefficients** for model interpretability
4. **Multi-scale analysis**: Check if short/medium/long features have different importance

### 10.2 For Pipeline

1. **Update `forensic:e2e`** to filter to 24 features by default
2. **Add symmetric validation** to feature computation
3. **Document feature pair semantics** in feature registry

### 10.3 For Future Work

1. **Adaptive lookback selection** based on volatility regime
2. **Feature importance by symmetric pair** (not individual features)
3. **Cross-validation across symbols** to verify generalization

---

## 11. Audit Metadata

**Audit Date**: 2026-01-25
**Auditor**: Claude Code (Symmetric Forensic Mode)
**Scope**: Feature selection with bull/bear symmetry preservation
**Principle**: Semantic correctness over statistical optimization

### Tasks Completed

| #   | Task                               | Status |
| --- | ---------------------------------- | ------ |
| 1   | Identify greedy selection failure  | PASS   |
| 2   | Define symmetric pairing principle | PASS   |
| 3   | Analyze threshold redundancy       | PASS   |
| 4   | Analyze lookback correlations      | PASS   |
| 5   | Validate 24-feature orthogonality  | PASS   |
| 6   | Document semantic interpretations  | PASS   |
| 7   | Generate symmetric audit report    | PASS   |

---

_Generated by symmetric forensic audit, 2026-01-25_
