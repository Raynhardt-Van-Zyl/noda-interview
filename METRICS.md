# Metrics

This file tracks baseline performance against the active optimization branches.
All values below were measured on Raynhardt's Arch Linux workstation.

## Benchmark Method

```text
Date: 2026-05-12
Fixture:
  examples/sample.csv: 65 MiB, 1,000,011 logical records
  examples/sample.ndjson: 101 MiB, 1,000,011 records
Build:
  cargo build --release
Run:
  target/release/noda-interview --batch-size 1000
Database:
  fresh temporary SQLite database per format/run
Peak RSS:
  sampled from /proc/<pid>/status VmHWM while the process ran
Binary size:
  stat -c %s target/release/noda-interview
```

The CLI reports rows per second from total processed input records. Filtered
rows are reported separately from failed rows because an empty tag is a normal
business filter, not a parsing or write failure.

## Measured Branch Revisions

| Branch | Commit | Purpose |
| --- | --- | --- |
| `main` | `87c597a` | Clean baseline. |
| `perf/single-transaction` | `ff9d025` | Use one SQLite transaction and prepared statement per run. |
| `perf/csv-byterecord` | `44922ed` | Parse CSV with reusable `csv::ByteRecord` instead of serde row deserialization. |
| `perf/ndjson-buffer` | `4a74fbd` | Reuse one buffer while reading NDJSON lines. |

## Outcome Counts

All measured branches produced the same row outcomes for both CSV and NDJSON.

| Metric | Value |
| --- | ---: |
| Total records processed | 1,000,011 |
| Successful rows written | 685,619 |
| Failed rows | 247,928 |
| Filtered empty tags | 66,464 |

## Summary

| Branch | Avg rows/sec | Delta vs base | Avg duration | Avg max RSS | RSS delta | Binary size | Size delta |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `main` | 211,112 | baseline | 4.737s | 7,628 KiB | baseline | 3,639,496 bytes | baseline |
| `perf/single-transaction` | 431,257 | +220,145 (+104.28%) | 2.319s | 7,674 KiB | +46 KiB (+0.60%) | 3,640,040 bytes | +544 bytes (+0.01%) |
| `perf/csv-byterecord` | 218,103 | +6,991 (+3.31%) | 4.585s | 7,640 KiB | +12 KiB (+0.16%) | 3,655,808 bytes | +16,312 bytes (+0.45%) |
| `perf/ndjson-buffer` | 217,979 | +6,867 (+3.25%) | 4.591s | 7,614 KiB | -14 KiB (-0.18%) | 3,638,160 bytes | -1,336 bytes (-0.04%) |

Chart labels:

```text
base = main
tx   = perf/single-transaction
csv  = perf/csv-byterecord
ndj  = perf/ndjson-buffer
```

## Graphs

```mermaid
xychart-beta
    title "Average Throughput"
    x-axis ["base", "tx", "csv", "ndj"]
    y-axis "rows/sec" 0 --> 450000
    bar [211112, 431257, 218103, 217979]
```

```mermaid
xychart-beta
    title "Average Duration"
    x-axis ["base", "tx", "csv", "ndj"]
    y-axis "seconds" 0 --> 5
    bar [4.737, 2.319, 4.585, 4.591]
```

```mermaid
xychart-beta
    title "Average Max RSS"
    x-axis ["base", "tx", "csv", "ndj"]
    y-axis "KiB" 0 --> 8000
    bar [7628, 7674, 7640, 7614]
```

```mermaid
xychart-beta
    title "Release Binary Size"
    x-axis ["base", "tx", "csv", "ndj"]
    y-axis "bytes" 0 --> 3700000
    bar [3639496, 3640040, 3655808, 3638160]
```

```mermaid
xychart-beta
    title "CSV Throughput"
    x-axis ["base", "tx", "csv", "ndj"]
    y-axis "rows/sec" 0 --> 450000
    bar [211363, 436661, 217488, 224214]
```

```mermaid
xychart-beta
    title "NDJSON Throughput"
    x-axis ["base", "tx", "csv", "ndj"]
    y-axis "rows/sec" 0 --> 450000
    bar [210861, 425854, 218718, 211743]
```

## Per-Format Speed

| Branch | Format | Duration | Rows/sec | Delta vs same-format base |
| --- | --- | ---: | ---: | ---: |
| `main` | CSV | 4.731s | 211,362.89 | baseline |
| `main` | NDJSON | 4.743s | 210,861.03 | baseline |
| `perf/single-transaction` | CSV | 2.290s | 436,660.84 | +225,298 (+106.59%) |
| `perf/single-transaction` | NDJSON | 2.348s | 425,853.84 | +214,993 (+101.96%) |
| `perf/csv-byterecord` | CSV | 4.598s | 217,488.01 | +6,125 (+2.90%) |
| `perf/csv-byterecord` | NDJSON | 4.572s | 218,717.91 | +7,857 (+3.73%) |
| `perf/ndjson-buffer` | CSV | 4.460s | 224,214.30 | +12,851 (+6.08%) |
| `perf/ndjson-buffer` | NDJSON | 4.723s | 211,742.96 | +882 (+0.42%) |

## Memory And Binary Size

| Branch | CSV max RSS | NDJSON max RSS | Release binary |
| --- | ---: | ---: | ---: |
| `main` | 7,636 KiB | 7,620 KiB | 3,639,496 bytes |
| `perf/single-transaction` | 7,680 KiB | 7,668 KiB | 3,640,040 bytes |
| `perf/csv-byterecord` | 7,668 KiB | 7,612 KiB | 3,655,808 bytes |
| `perf/ndjson-buffer` | 7,596 KiB | 7,632 KiB | 3,638,160 bytes |

## Notes

- `perf/single-transaction` is the clear high-impact speed win in this run,
  roughly doubling throughput with almost no memory or binary-size cost.
- `perf/csv-byterecord` has only a small standalone speed gain here, despite
  being useful in the older cumulative optimization chain.
- `perf/ndjson-buffer` has the smallest binary and memory footprint in this
  run, but its direct NDJSON speed improvement is small at this batch size.
- Memory values are low compared with older notes because this measurement uses
  the current clean baseline and sampled process `VmHWM` directly per run.
