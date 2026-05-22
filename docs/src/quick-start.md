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

Generate a local example fixture:

```bash
python3 examples/data_generator.py \
  --rows 100000 \
  --dirty \
  --csv target/fixtures/sample.csv \
  --ndjson target/fixtures/sample.ndjson
```

Run a CSV load:

```bash
cargo run --release -- \
  --input target/fixtures/sample.csv \
  --format csv \
  --db metrics.sqlite \
  --batch-size 1000 \
  --log-file target/fixtures/events.jsonl
```

Run an NDJSON load:

```bash
cargo run --release -- \
  --input target/fixtures/sample.ndjson \
  --format ndjson \
  --db metrics.sqlite
```

Use the same pipeline from Rust:

```rust,ignore
use noda_interview::{EtlConfig, cli::InputFormat, run_etl};

let config = EtlConfig::new(
    "target/fixtures/sample.ndjson",
    InputFormat::Ndjson,
    "metrics.sqlite",
)
.with_batch_size(1000)
.with_log_file("target/fixtures/events.jsonl");

let metrics = run_etl(&config)?;
println!("{}", metrics.summary());
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

When `--log-file` is supplied, parse failures, validation failures, filtered
empty tags, and duplicate-key write failures are written as JSON Lines. Each log
entry includes the failure stage, reason, source row number, raw source entry
when available, and the parsed or cleaned entry being handled.

Create a pretty JSON copy for manual inspection:

```bash
python3 - <<'PY'
import json
from pathlib import Path

source = Path("target/fixtures/events.jsonl")
target = Path("target/fixtures/events.pretty.json")
events = [json.loads(line) for line in source.read_text().splitlines() if line.strip()]
target.write_text(json.dumps(events, indent=2) + "\n")
PY
```
