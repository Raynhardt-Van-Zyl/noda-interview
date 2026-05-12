# Introduction

Noda Interview ETL is a small Rust project for loading event-like metrics into
SQLite. It accepts CSV or NDJSON input, transforms each record, and writes clean
rows into an existing `metrics` table.

The implementation focuses on three constraints:

- Stream input instead of reading entire files into memory.
- Keep parsing, transformation, and database writes separated.
- Batch SQLite writes while keeping row-level failures visible in the metrics.

The rest of this book documents how to run the loader, what input it accepts,
how rows are transformed, and how the test data was generated.
