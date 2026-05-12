# Code Structure

This is a small binary crate. The modules follow the same order as the ETL
pipeline, so the command-line entry point does not need to know CSV, JSON, or
SQLite details directly.

| File | Purpose |
| --- | --- |
| `src/main.rs` | CLI orchestration, batching, and final metric output. |
| `src/cli.rs` | Command-line argument parsing. |
| `src/input.rs` | Streaming CSV and NDJSON readers. |
| `src/model.rs` | Raw input and cleaned output record types. |
| `src/transform.rs` | Validation and normalization rules. |
| `src/db.rs` | SQLite connection and batch insertion. |
| `src/metrics.rs` | Runtime counters and summary formatting. |

The crate could be split into a reusable library later. For this assignment,
keeping the orchestration in `main.rs` makes the data flow easy to follow.
