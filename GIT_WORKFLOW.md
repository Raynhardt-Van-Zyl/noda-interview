# Git Workflow

This repository was organized so the interview baseline stays easy to review
while optimization experiments remain inspectable.

## Branches

| Branch | Purpose |
| --- | --- |
| `main` | Clean assignment implementation, documentation, tests, CI, and metrics. |
| `perf/single-transaction` | SQLite transaction-focused optimization. |
| `perf/csv-byterecord` | CSV parsing optimization experiment. |
| `perf/ndjson-buffer` | NDJSON line-buffer reuse experiment. |
| `perf/combined` | Combined optimization experiment. |

The intended submission branch is `main`. The performance branches are useful
for discussion, but they are not required to understand or run the assignment.

## Worktrees

The optimization branches were checked out as sibling worktrees. That made it
possible to build and benchmark branches sequentially without repeatedly
switching the main working directory.

```bash
git worktree list
```

The scale benchmark script expects this layout by default:

```text
noda-interview/                         main
noda-interview-worktrees/single-transaction
noda-interview-worktrees/csv-byterecord
noda-interview-worktrees/ndjson-buffer
noda-interview-worktrees/combined
```

## CI

GitLab CI is defined in `.gitlab-ci.yml` and has three stages:

| Stage | Checks |
| --- | --- |
| `verify` | `cargo fmt`, Clippy, tests, release build, and Rust docs. |
| `docs` | Build and test the mdBook documentation. |
| `performance` | Generate a deterministic 100k-row fixture and run a smoke performance gate. |

The performance gate is intentionally broad. It is there to catch obvious
regressions, not to replace local benchmarking.

## Benchmark Artifacts

Generated benchmark files live under `target/` and are intentionally ignored by
git. The committed benchmark results are summarized in `METRICS.md`; raw local
SQLite databases are temporary and should be deleted after runs.
