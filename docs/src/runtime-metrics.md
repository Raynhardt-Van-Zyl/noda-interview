# Runtime Metrics

Each run reports:

| Metric | Meaning |
| --- | --- |
| Total records processed | Raw input records read. |
| Successful rows written | Rows inserted into SQLite. |
| Failed rows | Invalid rows plus duplicate IDs. |
| Filtered empty tags | Rows skipped after tag normalization. |
| Total duration | Wall-clock pipeline runtime. |
| Rows per second | Total records divided by duration. |

Filtered rows are reported separately from failed rows because an empty tag is a
valid business filter, not a parsing or write failure.

The summary is intentionally compact. Use `--log-file` when you need row-level
details behind the counters. For example, `Failed rows: 20903` can be broken
down in the JSONL log by `stage` and `reason`, while `Filtered empty tags:
6572` can be traced back to exact source rows.
