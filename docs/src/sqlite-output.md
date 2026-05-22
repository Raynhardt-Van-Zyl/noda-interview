# SQLite Output

The database file and target table must exist before the loader starts. The CLI
opens the database in read/write mode without creating missing files, then
validates the target table schema.

```sql
CREATE TABLE metrics (
  id TEXT PRIMARY KEY,
  timestamp INTEGER NOT NULL,
  value REAL NOT NULL,
  tag TEXT NOT NULL,
  positive INTEGER NOT NULL
);
```

Clean records are inserted with:

```sql
INSERT INTO metrics (id, timestamp, value, tag, positive)
VALUES (?1, ?2, ?3, ?4, ?5);
```

Duplicate IDs do not abort the run. SQLite returns an insert error for the
conflicting row, the pipeline counts that row as failed, and the remaining rows
in the batch still run.

When structured logging is enabled, duplicate-key failures include both the
cleaned row attempted against SQLite and the original input context:

```json
{
  "event": "failed_row",
  "stage": "database",
  "reason": "UNIQUE constraint failed: metrics.id",
  "context": {
    "input_path": "target\\observability-check\\input.csv",
    "format": "csv",
    "row_number": 2,
    "raw": ["dup", "2026-05-11T00:00:01Z", "2.0", "prod"]
  },
  "entry": {
    "id": "dup",
    "timestamp": 1778457601,
    "value": 2.0,
    "tag": "prod",
    "positive": true
  }
}
```

Unexpected database errors, such as a missing table or incompatible schema, are
returned as fatal errors instead of being hidden inside the failed-row count.
