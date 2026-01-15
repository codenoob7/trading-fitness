# Trading Fitness

Polyglot monorepo for trading strategy fitness analysis using ITH (Investment Time Horizon) methodology.

## Quick Start

```bash
# Install dependencies
mise install

# Run ITH analysis on custom NAV data
mise run analyze

# View results
open artifacts/results.html
```

## Package Structure

| Package        | Language | Purpose                                 |
| -------------- | -------- | --------------------------------------- |
| `ith-python`   | Python   | ITH fitness analysis (PRIMARY)          |
| `core-rust`    | Rust     | Performance-critical code (placeholder) |
| `core-bun`     | Bun/TS   | Async I/O, APIs (placeholder)           |
| `shared-types` | Multi    | Cross-language schemas                  |

## Data Flow

```
data/nav_data_custom/*.csv  -->  [ith-python]  -->  artifacts/synth_ithes/
                                      |
                                      v
                              artifacts/results.html
```

## Input Format

Place CSV files with `Date` and `NAV` columns in `data/nav_data_custom/`:

```csv
Date,NAV
2024-01-01,100.00
2024-01-02,100.50
```

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [ITH Methodology](docs/ITH.md)
- [Logging Contract](docs/LOGGING.md)

## Tasks

```bash
mise run analyze     # Run ITH analysis
mise run test        # Run tests
mise run lint        # Lint all packages
mise run affected    # List affected packages
```
