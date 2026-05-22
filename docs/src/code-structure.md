# Code Structure

This is a library-first crate with a thin CLI wrapper. The public library API is
intended for embedding the loader in a larger Rust codebase, while `src/main.rs`
only parses command-line arguments, calls `run_etl`, and prints the returned
metrics.

| File | Purpose |
| --- | --- |
| `src/lib.rs` | Public crate API, `EtlConfig`, and `run_etl` orchestration. |
| `src/main.rs` | Thin CLI adapter around the library API. |
| `src/cli.rs` | Command-line argument parsing. |
| `src/input.rs` | Streaming CSV and NDJSON readers. |
| `src/model.rs` | Raw, cleaned, contextual, and prepared record types. |
| `src/transform.rs` | Validation and normalization rules. |
| `src/db.rs` | SQLite connection and batch insertion. |
| `src/event_log.rs` | Optional structured JSONL logging for failed and filtered rows. |
| `src/metrics.rs` | Runtime counters and summary formatting. |
| `tests/library_api.rs` | Direct coverage for the reusable Rust API. |

The top-level library contract is:

```rust,ignore
use noda_interview::{EtlConfig, run_etl};
```

Lower-level modules are public for targeted reuse or testing:

| Contract | Meaning |
| --- | --- |
| `EtlConfig` | Library-owned run configuration independent from Clap. |
| `run_etl` | Execute one ETL run and return `RunMetrics`. |
| `RawRecord` | Input shape before validation. |
| `RecordContext` | Source file format, row number, and raw entry. |
| `CleanRecord` | Validated row ready for SQLite. |
| `PreparedRecord` | `CleanRecord` plus `RecordContext` for batch-level diagnostics. |
| `BatchInsertResult` | Insert count plus row-level database failures. |
