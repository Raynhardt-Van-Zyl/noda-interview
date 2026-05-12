# Input Data Spoofing

The assignment requires an example dataset with more than 100k records. This
project uses a deterministic generator instead of hand-maintained large files,
so correctness and performance runs can be reproduced.

Generate CSV and NDJSON fixtures:

```bash
python3 examples/data_generator.py \
  --rows 100000 \
  --dirty \
  --csv target/fixtures/sample.csv \
  --ndjson target/fixtures/sample.ndjson
```

The generator emits the required fields:

```text
id, timestamp, value, tag
```

The default seed is fixed. Passing a different `--seed` creates a different,
but still reproducible, dataset. With `--dirty`, the command above writes
100,011 records because 11 deterministic edge rows are prepended to the
requested random rows.

## Edge Cases

The `--dirty` option mixes normal records with rows that exercise validation,
filtering, and database error handling.

| Case | Expected behavior |
| --- | --- |
| Positive values | Inserted with `positive = 1`. |
| Negative values | Inserted with `positive = 0`. |
| Zero and negative zero | Inserted with `positive = 0`. |
| Tags with leading/trailing whitespace | Trimmed before insert. |
| Mixed-case tags | Lowercased before insert. |
| Empty tags | Counted as filtered and not inserted. |
| Duplicate IDs | Counted as failed rows after SQLite rejects the primary key. |
| Invalid timestamps | Counted as failed rows during transformation. |
| Non-finite numeric values | Covered by targeted tests and optional `--nonstandard-json-floats` fixtures. |
| Malformed CSV or NDJSON rows | Covered by targeted integration tests. |
| Escaped strings, commas, tabs, and Unicode | Parsed by the format reader and normalized normally. |

Small integration tests use hand-written fixtures so failures stay readable.
The CI performance job uses generated 100k-row fixtures to exercise the same
code paths at the assignment scale.
