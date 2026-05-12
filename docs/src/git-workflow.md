# Git Workflow

The `main` branch is the assignment submission. It contains the implementation,
tests, documentation, CI setup, and benchmark summary.

Performance experiments were kept on separate branches so the submitted code
stays easy to review.

| Branch | Purpose |
| --- | --- |
| `main` | Assignment implementation. |
| `perf/single-transaction` | SQLite transaction-focused optimization. |
| `perf/csv-byterecord` | CSV parser optimization experiment. |
| `perf/ndjson-buffer` | NDJSON buffer reuse experiment. |
| `perf/combined` | Combined optimization experiment. |

## Worktrees

The optimization branches can be checked out as sibling worktrees:

```bash
git worktree list
```

The local benchmark script expects this layout by default:

```text
noda-interview/                         main
noda-interview-worktrees/single-transaction
noda-interview-worktrees/csv-byterecord
noda-interview-worktrees/ndjson-buffer
noda-interview-worktrees/combined
```

This keeps branch comparisons repeatable without repeatedly switching the main
working tree.

## GitLab CI

`.gitlab-ci.yml` runs three stages:

| Stage | Checks |
| --- | --- |
| `verify` | Formatting, Clippy, tests, release build, and Rust docs. |
| `docs` | mdBook build and mdBook tests. |
| `performance` | Deterministic 100k-row fixture and a broad throughput gate. |

Generated benchmark files live under `target/` and are ignored by git. The
committed benchmark summary lives in `METRICS.md`.
