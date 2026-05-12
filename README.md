# Noda Interview ETL

Baseline Rust ETL for streaming CSV or NDJSON records into an existing SQLite
`metrics` table.

This `main` branch intentionally represents the pre-optimization baseline. The
optimization branches are checked out as sibling worktrees so runtime behavior
can be compared without repeatedly switching branches.

## Features

- Streams CSV and NDJSON without loading the full file into memory.
- Normalizes timestamps, tags, and positivity flags.
- Writes clean rows to SQLite in configurable batches.
- Counts duplicate primary-key writes as failed rows.
- Reports row counts, duration, and throughput.
- Includes mdBook documentation, GitLab CI, and baseline metrics notes.

## Quick Start

Create the target SQLite table:

```sql
CREATE TABLE metrics (
  id TEXT PRIMARY KEY,
  timestamp INTEGER NOT NULL,
  value REAL NOT NULL,
  tag TEXT NOT NULL,
  positive INTEGER NOT NULL
);
```

Run the CLI:

```bash
cargo run --release -- \
  --input examples/sample.csv \
  --format csv \
  --db metrics.sqlite \
  --batch-size 1000
```

Use NDJSON by changing the input path and format:

```bash
cargo run --release -- \
  --input examples/sample.ndjson \
  --format ndjson \
  --db metrics.sqlite
```

## CLI

```text
--input <path>                 Input CSV or NDJSON file
--format <csv|ndjson>          Input file format
--db <path>                    SQLite database file
--batch-size <number>          Clean records per batch, default 1000
```

The SQLite database file and `metrics` table must already exist before running
the command.

## Input Shape

CSV input expects this header:

```csv
id,timestamp,value,tag
```

NDJSON input expects one JSON object per line:

```json
{"id":"event-1","timestamp":"2026-05-11T00:00:00Z","value":1.5,"tag":"Prod"}
```

Transformation rules:

```text
timestamp string -> Unix epoch seconds
tag.trim().to_lowercase()
empty tag after trim -> filtered out
positive = 1 when value > 0.0, otherwise 0
NaN or infinite values -> failed row
duplicate id -> failed row
```

## Documentation

- mdBook source: [docs/](docs/)
- Baseline metrics: [METRICS.md](METRICS.md)
- Input data spoofing and edge cases: [INPUT_DATA_SPOOFING.md](INPUT_DATA_SPOOFING.md)
- Git branch, worktree, and CI setup: [GIT_WORKFLOW.md](GIT_WORKFLOW.md)
- Build the book locally with `mdbook build docs`.

## Development

```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
cargo doc --no-deps
scripts/ci_performance.sh
```

GitLab CI runs formatting, Clippy, tests, Rust docs, mdBook validation, and a
generated performance regression smoke check.
