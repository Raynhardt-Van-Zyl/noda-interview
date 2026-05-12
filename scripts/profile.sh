#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  scripts/profile.sh <csv|ndjson> <flamegraph|perf> [batch-size]

Examples:
  scripts/profile.sh csv flamegraph
  scripts/profile.sh ndjson perf 1000

Artifacts are written to target/profiling/.
USAGE
}

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

if [[ $# -lt 2 || $# -gt 3 ]]; then
  usage
  exit 2
fi

format="$1"
mode="$2"
batch_size="${3:-1000}"

case "$format" in
  csv) input="examples/sample.csv" ;;
  ndjson) input="examples/sample.ndjson" ;;
  *)
    usage
    exit 2
    ;;
esac

case "$mode" in
  flamegraph|perf) ;;
  *)
    usage
    exit 2
    ;;
esac

require_command cargo
require_command sqlite3

mkdir -p target/profiling
db_path="$(mktemp "/tmp/noda-profile-${format}.XXXXXX.sqlite")"
trap 'rm -f "$db_path"' EXIT
create_metrics_table "$db_path"

case "$mode" in
  flamegraph)
    require_command perf
    if ! cargo flamegraph --help >/dev/null 2>&1; then
      printf 'missing cargo flamegraph subcommand; install cargo-flamegraph\n' >&2
      exit 127
    fi

    cargo flamegraph \
      --profile profiling \
      --bin noda-interview \
      --output "target/profiling/${format}.flamegraph.svg" \
      -- \
      --input "$input" \
      --format "$format" \
      --db "$db_path" \
      --batch-size "$batch_size"
    ;;
  perf)
    require_command perf
    cargo build --profile profiling --bin noda-interview

    perf record \
      --freq 997 \
      --call-graph dwarf \
      --output "target/profiling/${format}.perf.data" \
      -- \
      target/profiling/noda-interview \
      --input "$input" \
      --format "$format" \
      --db "$db_path" \
      --batch-size "$batch_size"

    perf report \
      --input "target/profiling/${format}.perf.data" \
      --stdio \
      --no-children \
      --sort comm,dso,symbol \
      >"target/profiling/${format}.perf.report.txt"
    ;;
esac
