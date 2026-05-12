# Pipeline Architecture

The pipeline has four layers:

1. Input streams raw records from CSV or NDJSON.
2. Transform validates and normalizes each raw record.
3. `main.rs` collects clean records into an in-process `Vec` batch.
4. SQLite inserts each batch into the existing `metrics` table.

`main.rs` owns the orchestration. When a batch reaches `--batch-size`, it is
flushed to SQLite and the same `Vec` is cleared for reuse.

Each flush opens one SQLite transaction and prepares the insert statement for
that batch.

The default batch size is `1000`.
