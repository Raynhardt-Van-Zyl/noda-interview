# Development

Common checks:

```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
cargo doc --no-deps
mdbook build docs
mdbook test docs
scripts/ci_performance.sh
```

Useful source entry points:

| File | Purpose |
| --- | --- |
| `src/main.rs` | CLI orchestration, batching, and final metric output. |
| `src/input.rs` | CSV and NDJSON streaming readers. |
| `src/transform.rs` | Validation and normalization. |
| `src/db.rs` | SQLite insertion path. |
| `src/metrics.rs` | Runtime measurement helpers. |

## CI

GitLab CI runs three stages:

| Stage | Purpose |
| --- | --- |
| `verify` | Formatting, Clippy, tests, release build, and Rust docs. |
| `docs` | mdBook build and mdBook tests. |
| `performance` | 100k-row generated fixture with a broad throughput gate. |
