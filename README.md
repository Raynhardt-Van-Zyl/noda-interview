# Noda Interview ETL

Rust CLI for streaming CSV or NDJSON records, transforming them, and writing
clean rows into an existing SQLite `metrics` table.

## Current Status

Implemented streaming ETL flow for CSV and NDJSON inputs.

## CLI

```bash
cargo run -- \
  --input examples/sample.csv \
  --format csv \
  --db metrics.sqlite \
  --batch-size 1000
```

## Flags

```text
--input <path>
--format <csv|ndjson>
--db <path>
--batch-size <number, default 1000>
```

## SQLite Schema

```sql
CREATE TABLE metrics (
  id TEXT PRIMARY KEY,
  timestamp INTEGER NOT NULL,
  value REAL NOT NULL,
  tag TEXT NOT NULL,
  positive INTEGER NOT NULL
);
```

## Project Layout

```text
src/
  main.rs
  cli.rs
  model.rs
  input.rs
  transform.rs
  db.rs
  metrics.rs

tests/
  integration.rs

examples/
  sample.csv
  sample.ndjson
```

## Transformation Rules

```text
timestamp string -> Unix epoch seconds
tag.trim().to_lowercase()
empty tag after trim -> filtered out
positive = 1 when value > 0.0, otherwise 0
NaN or infinite values -> failed row
```

## Metrics

```text
Total records processed:
Successful rows written:
Failed rows:
Filtered empty tags:
Total duration:
Rows per second:
```

## Performance Log

Baseline machine: Raynhardt's Arch Linux workstation. Commands were run with
`cargo run --release` against a temporary SQLite database on `/tmp`, with the
existing `metrics` table created before each run.

Fixture commit: `9fc8fd0` (`Regenerate million-row sample fixtures`)

```text
Fixture:
  examples/sample.csv: 65M, 1,000,011 logical records
  examples/sample.ndjson: 101M, 1,000,011 records

Batch size:
  10000

CSV baseline:
  Total records processed: 1000011
  Successful rows written: 685619
  Failed rows: 247928
  Filtered empty tags: 66464
  Total duration: 3.262s
  Rows per second: 306542.79

NDJSON baseline:
  Total records processed: 1000011
  Successful rows written: 685619
  Failed rows: 247928
  Filtered empty tags: 66464
  Total duration: 2.909s
  Rows per second: 343796.34
```

## Implementation Decisions

```text
Input files are streamed and never collected into a Vec.
Duplicate primary keys are counted as failed rows.
Rows per second is calculated from total processed rows.
Filtered rows are reported separately from failed rows.
The SQLite database and `metrics` table must already exist before running the CLI.
```
