# Input Formats

Both input formats map into the same raw record shape:

| Field | Type | Notes |
| --- | --- | --- |
| `id` | string | Used as the SQLite primary key. |
| `timestamp` | string | Must parse as RFC 3339. |
| `value` | number | Must be finite. |
| `tag` | string | Trimmed and lowercased during transformation. |

## CSV

CSV input expects this header:

```csv
id,timestamp,value,tag
```

Rows are read through `csv::Reader::records()`, then deserialized with the
captured header row. The pipeline keeps a small `RecordContext` beside each
parsed record so later validation or database failures can still point back to
the original CSV row number and raw field values.

## NDJSON

NDJSON input expects one JSON object per line:

```json
{"id":"event-1","timestamp":"2026-05-11T00:00:00Z","value":1.5,"tag":"Prod"}
```

Lines are read through `BufReader::lines()` and parsed with
`serde_json::from_str`. This keeps the input streaming even though each line is
allocated while it is parsed. Malformed JSON lines are reported to the caller
with the raw line text so they can be written to the structured debug log.

## Transformation Rules

```text
timestamp string -> Unix epoch seconds
tag.trim().to_lowercase()
empty tag after trim -> filtered out
positive = 1 when value > 0.0, otherwise 0
NaN or infinite values -> failed row
duplicate id -> failed row
```

Rows are handled independently. A parse error, invalid timestamp, non-finite
value, empty tag, or duplicate primary key does not stop the rest of the input
file from being processed.
