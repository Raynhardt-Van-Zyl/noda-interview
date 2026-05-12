# Baseline Metrics

This file tracks the baseline branch metrics. Optimization branches keep their
own benchmark outputs in their worktrees and can later be concatenated into a
comparison report from this baseline worktree.

Baseline machine: Raynhardt's Arch Linux workstation. Commands were run with
`cargo run --release` against a temporary SQLite database on `/tmp`, with the
existing `metrics` table created before each run.

Fixture commit: `9fc8fd0` (`Regenerate million-row sample fixtures`)

## Runtime Summary Fields

```text
Total records processed:
Successful rows written:
Failed rows:
Filtered empty tags:
Total duration:
Rows per second:
```

Rows per second is calculated from total processed rows. Filtered rows are
reported separately from failed rows.

## Million-Row Baseline

```text
Fixture:
  examples/sample.csv: 65M, 1,000,011 logical records
  examples/sample.ndjson: 101M, 1,000,011 records

Batch size:
  10000

CSV baseline:
  Total records processed: 1000011
  Successful rows written: 685619
  Failed rows: 247928
  Filtered empty tags: 66464
  Total duration: 3.262s
  Rows per second: 306542.79

NDJSON baseline:
  Total records processed: 1000011
  Successful rows written: 685619
  Failed rows: 247928
  Filtered empty tags: 66464
  Total duration: 2.909s
  Rows per second: 343796.34
```

## Benchmark Worktree Layout

The benchmark worktrees live beside this repository:

```text
/home/raynhardt/Documents/noda-tech/noda-interview-worktrees/
```

Shared large fixtures should live outside all worktrees:

```text
/home/raynhardt/Documents/noda-tech/noda-interview-bench-data/
```

Each worktree writes raw benchmark output under its own ignored
`target/bench-*/results/` directory.
