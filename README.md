# Noda Interview ETL

Rust ETL tool for streaming CSV or NDJSON records into an existing SQLite
`metrics` table.

## Features

- Streams CSV and NDJSON without loading the full file into memory.
- Normalizes timestamps, tags, and positivity flags.
- Writes clean rows to SQLite in configurable batches.
- Validates that the SQLite database and target table already exist.
- Counts duplicate primary-key writes as failed rows.
- Reports row counts, duration, and throughput.
- Documents usage, generated fixtures, CI checks, and benchmark results.

## Quick Start

Create the target SQLite database and table:

```bash
sqlite3 metrics.sqlite <<'SQL'
CREATE TABLE metrics (
  id TEXT PRIMARY KEY,
  timestamp INTEGER NOT NULL,
  value REAL NOT NULL,
  tag TEXT NOT NULL,
  positive INTEGER NOT NULL
);
SQL
```

Generate a local example fixture:

```bash
python3 examples/data_generator.py \
  --rows 100000 \
  --dirty \
  --csv target/fixtures/sample.csv \
  --ndjson target/fixtures/sample.ndjson
```

Run the CLI:

```bash
cargo run --release -- \
  --input target/fixtures/sample.csv \
  --format csv \
  --db metrics.sqlite \
  --batch-size 1000
```

Use NDJSON by changing the input path and format:

```bash
cargo run --release -- \
  --input target/fixtures/sample.ndjson \
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

`Filtered empty tags` are reported separately from `Failed rows`: they are
valid business skips, not malformed input or database failures.

## Documentation

- mdBook source: [docs/](docs/)
- Baseline metrics: [METRICS.md](METRICS.md)
- Test data generation and edge cases: [INPUT_DATA_SPOOFING.md](INPUT_DATA_SPOOFING.md)
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
