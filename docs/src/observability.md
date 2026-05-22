# Observability and Debug Logs

The loader prints aggregate runtime metrics to stdout and can also write
row-level JSON Lines events with `--log-file`.

```bash
cargo run --release -- \
  --input target/fixtures/sample.csv \
  --format csv \
  --db target/fixtures/metrics.sqlite \
  --batch-size 1000 \
  --log-file target/fixtures/events.jsonl
```

Library callers enable the same log file through `EtlConfig`:

```rust,ignore
use noda_interview::{EtlConfig, cli::InputFormat};

let config = EtlConfig::new("input.csv", InputFormat::Csv, "metrics.sqlite")
    .with_log_file("events.jsonl");
```

The log file is optional. When omitted, the pipeline still counts failed and
filtered rows, but it does not write per-row diagnostics.

## Event Shape

Each line is one JSON object:

```json
{
  "event": "failed_row",
  "stage": "database",
  "reason": "UNIQUE constraint failed: metrics.id",
  "context": {
    "input_path": "target\\fixtures\\sample.csv",
    "format": "csv",
    "row_number": 42,
    "raw": ["dup", "2026-05-11T00:00:01Z", "2.0", "prod"]
  },
  "entry": {
    "id": "dup",
    "timestamp": 1778457601,
    "value": 2.0,
    "tag": "prod",
    "positive": true
  }
}
```

| Field | Meaning |
| --- | --- |
| `event` | `failed_row` or `filtered_empty_tag`. |
| `stage` | Pipeline stage that produced the event: `parse`, `transform`, or `database`. |
| `reason` | Human-readable failure or filter reason. |
| `context.input_path` | Source input path passed to the CLI. |
| `context.format` | `csv` or `ndjson`. |
| `context.row_number` | One-based physical input row number, excluding the CSV header. |
| `context.raw` | Original CSV fields or raw NDJSON line when available. |
| `entry` | Parsed `RawRecord` for transform events or `CleanRecord` for database events. |

## Event Types

| Event | Stage | Typical reason |
| --- | --- | --- |
| `failed_row` | `parse` | Malformed CSV field or invalid JSON line. |
| `failed_row` | `transform` | Invalid timestamp or non-finite `value`. |
| `failed_row` | `database` | Duplicate primary key. |
| `filtered_empty_tag` | `transform` | `tag` is empty after trimming whitespace. |

Fatal setup errors, such as a missing database file or incompatible table
schema, are returned as command failures instead of being written as row-level
events.

## Inspecting Logs

JSONL is efficient for streaming and shell tooling, but not pleasant to read in
an editor. Convert it to a pretty JSON array for manual inspection:

```bash
python3 - <<'PY'
import json
from pathlib import Path

source = Path("target/fixtures/events.jsonl")
target = Path("target/fixtures/events.pretty.json")
events = [json.loads(line) for line in source.read_text().splitlines() if line.strip()]
target.write_text(json.dumps(events, indent=2, ensure_ascii=False) + "\n")
PY
```

Summarize reasons without loading the pretty file:

```bash
python3 - <<'PY'
import json
from collections import Counter
from pathlib import Path

counts = Counter()
for line in Path("target/fixtures/events.jsonl").open(encoding="utf-8"):
    event = json.loads(line)
    counts[(event["event"], event["stage"], event["reason"])] += 1

for (event, stage, reason), count in counts.most_common():
    print(f"{count:8} {event:20} {stage:10} {reason}")
PY
```

## Design Notes

The input layer attaches `RecordContext` as soon as it reads a physical row.
After transformation, clean rows are batched as `PreparedRecord` values rather
than plain `CleanRecord` values. That keeps source context available even when a
database constraint failure occurs after batching.

This is the important observability boundary: every expected row-level failure
should explain both what the pipeline saw at the source and what stage rejected
it.
