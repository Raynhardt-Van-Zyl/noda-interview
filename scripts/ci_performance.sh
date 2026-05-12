#!/usr/bin/env bash
set -euo pipefail

fixture_dir="target/ci-fixtures"
artifact_dir="target/ci-performance"
rows="100000"
expected_total="100011"
expected_success="72532"
expected_failed="20906"
expected_filtered="6573"
min_rows_per_second="150000"

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    printf 'missing required command: %s\n' "$1" >&2
    exit 127
  fi
}

create_metrics_table() {
  sqlite3 "$1" '
    CREATE TABLE metrics (
      id TEXT PRIMARY KEY,
      timestamp INTEGER NOT NULL,
      value REAL NOT NULL,
      tag TEXT NOT NULL,
      positive INTEGER NOT NULL
    );
  '
}

metric_value() {
  local label="$1"
  local file="$2"

  awk -F ': ' -v label="$label" '$1 == label { print $2; exit }' "$file"
}

metric_number() {
  local label="$1"
  local file="$2"

  metric_value "$label" "$file" | awk '{ print $1 }'
}

assert_eq() {
  local label="$1"
  local actual="$2"
  local expected="$3"

  if [[ "$actual" != "$expected" ]]; then
    printf '%s expected %s but got %s\n' "$label" "$expected" "$actual" >&2
    exit 1
  fi
}

assert_float_at_least() {
  local label="$1"
  local actual="$2"
  local minimum="$3"

  awk -v actual="$actual" -v minimum="$minimum" -v label="$label" '
    BEGIN {
      if (actual + 0 < minimum + 0) {
        printf "%s expected at least %s but got %s\n", label, minimum, actual > "/dev/stderr"
        exit 1
      }
    }
  '
}

run_format() {
  local format="$1"
  local output="$artifact_dir/${format}.out"
  local db_path

  db_path="$(mktemp "/tmp/noda-ci-${format}.XXXXXX.sqlite")"
  trap 'rm -f "$db_path"' RETURN
  create_metrics_table "$db_path"

  target/release/noda-interview \
    --input "$fixture_dir/perf.${format}" \
    --format "$format" \
    --db "$db_path" \
    --batch-size 1000 \
    | tee "$output"

  assert_eq "$format total records" "$(metric_number "Total records processed" "$output")" "$expected_total"
  assert_eq "$format successful rows" "$(metric_number "Successful rows written" "$output")" "$expected_success"
  assert_eq "$format failed rows" "$(metric_number "Failed rows" "$output")" "$expected_failed"
  assert_eq "$format filtered rows" "$(metric_number "Filtered empty tags" "$output")" "$expected_filtered"
  assert_float_at_least "$format rows/sec" "$(metric_number "Rows per second" "$output")" "$min_rows_per_second"

  printf '%s\t%s\t%s\n' "$format" "rows_per_second" "$(metric_number "Rows per second" "$output")" >>"$artifact_dir/metrics.tsv"
}

require_command cargo
require_command python3
require_command sqlite3

mkdir -p "$fixture_dir" "$artifact_dir"
: >"$artifact_dir/metrics.tsv"

python3 examples/data_generator.py \
  --rows "$rows" \
  --dirty \
  --csv "$fixture_dir/perf.csv" \
  --ndjson "$fixture_dir/perf.ndjson"

cargo build --release

run_format csv
run_format ndjson
