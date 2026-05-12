# Introduction

Noda Interview ETL is a small Rust project for loading event-like metrics into
SQLite. It accepts CSV or NDJSON input, transforms each record, and writes clean
rows into an existing `metrics` table.

The current implementation focuses on three constraints:

- Stream input instead of reading entire files into memory.
- Keep the implementation small enough to understand before optimization.
- Batch SQLite writes while preserving the original baseline behavior.

The project is not published on crates.io yet. This book, the metrics file, and
the CI setup are groundwork for making future publishing and optimization work
cleaner.
