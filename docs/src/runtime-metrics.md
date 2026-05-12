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
