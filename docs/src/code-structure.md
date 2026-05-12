# Code Structure

This branch is intentionally a simple binary crate. It keeps the baseline easy
to read before comparing it with the optimization worktrees.

| File | Purpose |
| --- | --- |
| `src/main.rs` | CLI orchestration, batching, and final metric output. |
| `src/cli.rs` | Command-line argument parsing. |
| `src/input.rs` | Streaming CSV and NDJSON readers. |
| `src/model.rs` | Raw input and cleaned output record types. |
| `src/transform.rs` | Validation and normalization rules. |
| `src/db.rs` | SQLite connection and batch insertion. |
| `src/metrics.rs` | Runtime counters and summary formatting. |

The crate can be split into a reusable library before publishing, but the
baseline keeps all orchestration in `main.rs` so the original design remains
obvious.
