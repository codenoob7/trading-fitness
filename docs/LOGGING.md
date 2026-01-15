# Logging Contract

## Overview

All packages in trading-fitness emit structured NDJSON logs to `logs/` directory.

## Format

Each log line is a valid JSON object with required fields:

```json
{
  "timestamp": "2025-01-15T09:30:00.000Z",
  "level": "INFO",
  "message": "Processing started",
  "package": "ith-python"
}
```

## Required Fields

| Field       | Type     | Description                 |
| ----------- | -------- | --------------------------- |
| `timestamp` | ISO 8601 | UTC timestamp               |
| `level`     | string   | DEBUG, INFO, WARNING, ERROR |
| `message`   | string   | Human-readable message      |
| `package`   | string   | Source package name         |

## Optional Fields

| Field         | Type   | Description                    |
| ------------- | ------ | ------------------------------ |
| `context`     | object | Additional structured data     |
| `error`       | object | Error details with stack trace |
| `duration_ms` | number | Operation duration             |

## Log Levels

- **DEBUG**: Detailed diagnostic information
- **INFO**: General operational events
- **WARNING**: Potential issues, non-fatal
- **ERROR**: Failures requiring attention

## File Naming

```
logs/{package}-{date}.jsonl
```

Example: `logs/ith-python-2025-01-15.jsonl`

## Python (loguru)

```python
from loguru import logger
logger.add("logs/ith-python-{time:YYYY-MM-DD}.jsonl", serialize=True)
```

## Rust (tracing)

```rust
use tracing_subscriber::fmt::format::json;
tracing_subscriber::fmt().json().init();
```

## Bun (pino)

```typescript
import pino from "pino";
const logger = pino({
  transport: {
    target: "pino/file",
    options: { destination: "./logs/core-bun.jsonl" },
  },
});
```
