# Pipeline Architecture

The pipeline has five operational layers:

1. Input streams raw records from CSV or NDJSON.
2. Transform validates and normalizes each raw record.
3. `main.rs` attaches source context to clean records and collects them into an
   in-process `Vec` batch.
4. SQLite inserts each batch into the existing `metrics` table.
5. Event logging writes structured JSONL records for skipped or failed rows
   when `--log-file` is configured.

`run_etl` in `src/lib.rs` owns the orchestration. The CLI delegates to that
library function rather than implementing a separate path. When a batch reaches
`--batch-size`, it is flushed to SQLite and the same `Vec` is cleared for reuse.
Each batch element is a `PreparedRecord`, which pairs the `CleanRecord` with its
`RecordContext`. That pairing is what lets duplicate-ID database failures still
report the source row number and raw input fields.

Each flush opens one SQLite transaction and prepares the insert statement for
that batch.

The default batch size is `1000`.

## Failure Handling

The loader separates fatal run errors from row-level errors:

| Failure | Outcome |
| --- | --- |
| Input file cannot be opened | Fatal error |
| SQLite file missing | Fatal error |
| `metrics` table missing required columns | Fatal error |
| Extra `NOT NULL` SQLite column without a default | Fatal error |
| Malformed CSV/NDJSON row | Failed row, logged when enabled |
| Invalid timestamp | Failed row, logged when enabled |
| Non-finite numeric value | Failed row, logged when enabled |
| Empty tag after trim | Filtered row, logged when enabled |
| Duplicate primary key | Failed row, logged when enabled |

This keeps expected data-quality problems visible without letting one bad row
abort a long streaming run.
