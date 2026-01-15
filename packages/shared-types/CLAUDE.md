# shared-types

Cross-language type definitions for trading-fitness packages.

## Status

Initialized with schema directory. Ready for type definitions when needed.

## Purpose

- Define shared data structures across Python, Rust, and TypeScript
- JSON Schema for validation
- Code generation for type-safe interfaces

## Structure

```
shared-types/
└── schemas/
    └── .gitkeep
```

## Planned Schemas

| Schema                 | Purpose                  |
| ---------------------- | ------------------------ |
| `nav-record.json`      | NAV data record format   |
| `ith-result.json`      | ITH analysis result      |
| `fitness-metrics.json` | Strategy fitness metrics |

## Usage

Schemas will be consumed by:

- Python: pydantic models generated from JSON Schema
- Rust: serde structs generated from JSON Schema
- TypeScript: zod schemas generated from JSON Schema
