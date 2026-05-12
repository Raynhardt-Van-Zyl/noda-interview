#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
worktree_root="${NODA_WORKTREE_ROOT:-$(dirname "$repo_root")/noda-interview-worktrees}"
bench_root="${NODA_BENCH_ROOT:-$repo_root/target/scale-bench}"
scales="${NODA_BENCH_SCALES:-10000 100000 1000000 10000000}"
branches="${NODA_BENCH_BRANCHES:-main perf/single-transaction perf/csv-byterecord perf/ndjson-buffer perf/combined}"
formats="${NODA_BENCH_FORMATS:-csv ndjson}"
cleanup_fixtures="${NODA_BENCH_CLEANUP_FIXTURES:-0}"

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    printf 'missing required command: %s\n' "$1" >&2
    exit 127
  fi
}

worktree_for_branch() {
  case "$1" in
    main) printf '%s\n' "$repo_root" ;;
    perf/single-transaction) printf '%s\n' "$worktree_root/single-transaction" ;;
    perf/csv-byterecord) printf '%s\n' "$worktree_root/csv-byterecord" ;;
    perf/ndjson-buffer) printf '%s\n' "$worktree_root/ndjson-buffer" ;;
    perf/combined) printf '%s\n' "$worktree_root/combined" ;;
    *)
      printf 'unknown benchmark branch: %s\n' "$1" >&2
      exit 2
      ;;
  esac
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

metric_number() {
  local label="$1"
  local file="$2"

  awk -F ': ' -v label="$label" '$1 == label { print $2; exit }' "$file" | awk '{ print $1 }'
}

generate_fixture() {
  local scale="$1"
  local fixture_dir="$bench_root/fixtures/$scale"
  local csv_path="$fixture_dir/perf.csv"
  local ndjson_path="$fixture_dir/perf.ndjson"

  if [[ -s "$csv_path" && -s "$ndjson_path" ]]; then
    return
  fi

  mkdir -p "$fixture_dir"
  python3 "$repo_root/examples/data_generator.py" \
    --rows "$scale" \
    --dirty \
    --csv "$csv_path" \
    --ndjson "$ndjson_path"
}

build_branch() {
  local branch="$1"
  local worktree

  worktree="$(worktree_for_branch "$branch")"
  cargo build --release --manifest-path "$worktree/Cargo.toml"
}

run_one() {
  local scale="$1"
  local branch="$2"
  local format="$3"
  local worktree input_path output_path db_path max_rss pid rss commit binary_size

  worktree="$(worktree_for_branch "$branch")"
  input_path="$bench_root/fixtures/$scale/perf.$format"
  output_path="$bench_root/results/${scale}-${branch//\//-}-${format}.out"
  db_path="$bench_root/db/${scale}-${branch//\//-}-${format}.sqlite"
  commit="$(git -C "$worktree" rev-parse --short HEAD)"
  binary_size="$(stat -c %s "$worktree/target/release/noda-interview")"

  mkdir -p "$bench_root/results" "$bench_root/db"
  rm -f "$db_path"
  create_metrics_table "$db_path"

  max_rss=0
  "$worktree/target/release/noda-interview" \
    --input "$input_path" \
    --format "$format" \
    --db "$db_path" \
    --batch-size 1000 \
    >"$output_path" &
  pid=$!

  while kill -0 "$pid" 2>/dev/null; do
    if [[ -r "/proc/$pid/status" ]]; then
      rss="$(awk '/^VmHWM:/ { print $2; exit }' "/proc/$pid/status" 2>/dev/null || true)"
      if [[ -n "${rss:-}" && "$rss" -gt "$max_rss" ]]; then
        max_rss="$rss"
      fi
    fi
    sleep 0.005
  done
  wait "$pid"

  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$scale" "$branch" "$commit" "$format" \
    "$(metric_number 'Rows per second' "$output_path")" \
    "$(metric_number 'Total duration' "$output_path" | sed 's/s$//')" \
    "$(metric_number 'Total records processed' "$output_path")" \
    "$(metric_number 'Successful rows written' "$output_path")" \
    "$(metric_number 'Failed rows' "$output_path")" \
    "$(metric_number 'Filtered empty tags' "$output_path")" \
    "$max_rss" \
    "$binary_size" >>"$bench_root/results.tsv"

  rm -f "$db_path"
}

require_command cargo
require_command python3
require_command sqlite3

mkdir -p "$bench_root"
if [[ ! -f "$bench_root/results.tsv" ]]; then
  printf 'scale\tbranch\tcommit\tformat\trows_per_sec\tduration_s\ttotal_records\tsuccessful_rows\tfailed_rows\tfiltered_rows\tmax_rss_kib\tbinary_size_bytes\n' >"$bench_root/results.tsv"
fi

for branch in $branches; do
  build_branch "$branch"
done

for scale in $scales; do
  generate_fixture "$scale"
  for branch in $branches; do
    for format in $formats; do
      run_one "$scale" "$branch" "$format"
    done
  done

  if [[ "$cleanup_fixtures" == "1" ]]; then
    rm -rf "$bench_root/fixtures/$scale"
  fi
done

column -t -s $'\t' "$bench_root/results.tsv" >"$bench_root/results.txt"
cat "$bench_root/results.txt"
