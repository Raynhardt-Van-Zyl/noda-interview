# Input Data Spoofing

The assignment asks for an example dataset with more than 100k records. This
project includes a generator so the same edge cases can be reproduced instead
of hand-maintained in static files.

## Generator

```bash
python3 examples/data_generator.py \
  --rows 100000 \
  --dirty \
  --csv target/fixtures/sample.csv \
  --ndjson target/fixtures/sample.ndjson
```

The generator writes both CSV and NDJSON with the required assignment fields:

```text
id, timestamp, value, tag
```

The default seed is fixed, so generated fixtures are deterministic unless a
different `--seed` is provided. With `--dirty`, the command above writes
100,011 records because 11 deterministic edge rows are prepended to the
requested random rows.

## Covered Edge Cases

The `--dirty` option intentionally mixes normal records with rows that exercise
the ETL error-handling path.

| Case | Purpose | Expected behavior |
| --- | --- | --- |
| Valid positive values | Normal happy path | Inserted with `positive = 1`. |
| Valid negative values | Derived field check | Inserted with `positive = 0`. |
| Zero and negative zero | Boundary value check | Inserted with `positive = 0`. |
| Leading/trailing tag whitespace | Tag normalization | Trimmed before insert. |
| Uppercase/mixed-case tags | Tag normalization | Lowercased before insert. |
| Empty tags | Business filter | Counted as filtered, not inserted. |
| Duplicate IDs | SQLite primary-key handling | Counted as failed rows. |
| Invalid timestamps | Transform validation | Counted as failed rows. |
| Non-finite numeric values | Value validation | Covered by targeted tests and optional `--nonstandard-json-floats` fixtures. |
| Malformed CSV/NDJSON rows | Parser error handling | Covered by targeted integration tests. |
| Tags with quotes, commas, tabs, and Unicode | Format escaping and parsing | Inserted after normal normalization. |

## Why This Matters

The generated data checks the assignment requirements directly:

- both supported input formats are exercised with the same logical data shape;
- the file is streamed, so fixture size can be increased without changing code;
- database uniqueness errors are separated from business filters;
- parser and transform failures are counted without stopping the whole run.

The integration tests keep small hand-written fixtures for readability, while
the CI performance script uses the generator to create a larger 100k-row sample.
