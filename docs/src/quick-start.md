# Quick Start

Create the SQLite database and table before running the loader:

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

Run a CSV load:

```bash
cargo run --release -- \
  --input examples/sample.csv \
  --format csv \
  --db metrics.sqlite \
  --batch-size 1000
```

Run an NDJSON load:

```bash
cargo run --release -- \
  --input examples/sample.ndjson \
  --format ndjson \
  --db metrics.sqlite
```

The command prints a summary with row counts, duration, and throughput. Duration
and rows/sec vary by machine and input size.

```text
Total records processed: 1000011
Successful rows written: 685619
Failed rows: 247928
Filtered empty tags: 66464
Total duration: ...
Rows per second: ...
```
