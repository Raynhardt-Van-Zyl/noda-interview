#!/usr/bin/env python3
"""Generate CSV and NDJSON sample data for ETL testing.
"""

from __future__ import annotations

import argparse
import csv
import json
import math
import random
import string
import uuid
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any

FIELDNAMES = ("id", "timestamp", "value", "tag")

# made up random tags, but also added some weird stuff like unicode and emoji -_'
TAGS = (
    "alpha",
    "beta",
    "gamma",
    "prod",
    "staging",
    "sensor-a",
    "sensor,b",
    'quoted-"tag"',
    "leading space",
    "trailing space ",
    "tab\tseparated",
    "line\nbreak",
    "",
    "unicode-\u03c0",
    "emoji-\U0001f680",
)

FINITE_EDGE_VALUES = (
    0.0,
    -0.0,
    1.0,
    -1.0,
    0.000001,
    -0.000001,
    1e-12,
    -1e-12,
    1e12,
    -1e12,
    3.141592653589793,
    2.2250738585072014e-308,
    1.7976931348623157e308,
)

NONSTANDARD_FLOATS = (math.nan, math.inf, -math.inf)

# just made some fancy python cli parameters for the script, but probably pointless as
# this will only be used once to generate all the data
def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate randomized ETL fixture data in CSV and NDJSON formats."
    )
    parser.add_argument("-n", "--rows", type=int, default=100, help="rows to generate")
    parser.add_argument("--seed", type=int, default=42, help="random seed")
    parser.add_argument("--csv", type=Path, default=Path("sample.csv"), help="CSV output path")
    parser.add_argument(
        "--ndjson",
        type=Path,
        default=Path("sample.ndjson"),
        help="NDJSON output path",
    )
    parser.add_argument(
        "--dirty",
        action="store_true",
        help="include rows with empty fields, duplicate IDs, invalid timestamps, and other validation cases",
    )
    parser.add_argument(
        "--nonstandard-json-floats",
        action="store_true",
        help="with --dirty, allow NaN/Infinity in NDJSON for strict-parser rejection tests",
    )
    parser.add_argument(
        "--start",
        default="2026-01-01T00:00:00+00:00",
        help="base ISO-8601 timestamp for generated rows",
    )
    return parser.parse_args()

#overkill of a random id generator, used a bit of AI for this beast of point bloat 
def random_id(rng: random.Random) -> str:
    style = rng.choice(("uuid", "short", "numeric", "prefixed", "edge_chars"))
    if style == "uuid":
        return str(uuid.UUID(int=rng.getrandbits(128)))
    if style == "short":
        return "".join(rng.choices(string.ascii_letters + string.digits, k=rng.randint(1, 12)))
    if style == "numeric":
        return str(rng.randint(0, 1_000_000))
    if style == "prefixed":
        return f"event-{rng.randint(1, 999_999):06d}"
    return rng.choice((" id-with-spaces ", "id,comma", 'id"quote', "id/slash", "id:colon"))

#ISO-8601 is not very standard IMO 
def random_timestamp(rng: random.Random, base: datetime) -> str:
    offset_seconds = rng.randint(-365 * 24 * 3600, 365 * 24 * 3600)
    microseconds = rng.choice((0, rng.randint(1, 999_999)))
    dt = base + timedelta(seconds=offset_seconds, microseconds=microseconds)

    variant = rng.choice(("zulu", "offset", "millis", "date_boundary"))
    if variant == "zulu":
        return dt.astimezone(timezone.utc).isoformat().replace("+00:00", "Z")
    if variant == "offset":
        offset = timezone(timedelta(hours=rng.choice((-12, -5, 0, 2, 5, 14))))
        return dt.astimezone(offset).isoformat()
    if variant == "millis":
        return dt.astimezone(timezone.utc).isoformat(timespec="milliseconds").replace("+00:00", "Z")
    boundary = rng.choice(
        (
            datetime(1970, 1, 1, tzinfo=timezone.utc),
            datetime(1999, 12, 31, 23, 59, 59, 999999, tzinfo=timezone.utc),
            datetime(2038, 1, 19, 3, 14, 7, tzinfo=timezone.utc),
            datetime(9999, 12, 31, 23, 59, 59, 999999, tzinfo=timezone.utc),
        )
    )
    return boundary.isoformat().replace("+00:00", "Z")


# added more variants to make the random value more extreme sometimes 
def random_value(rng: random.Random) -> float:
    variant = rng.choice(("normal", "edge", "scientific", "rounded"))
    if variant == "edge":
        return rng.choice(FINITE_EDGE_VALUES)
    if variant == "scientific":
        return rng.choice((-1.0, 1.0)) * 10 ** rng.uniform(-10, 10)
    if variant == "rounded":
        return round(rng.uniform(-1_000_000, 1_000_000), rng.randint(0, 6))
    return rng.uniform(-10_000, 10_000)

# generation of the actual row  
def random_row(rng: random.Random, base: datetime) -> dict[str, Any]:
    return {
        "id": random_id(rng),
        "timestamp": random_timestamp(rng, base),
        "value": random_value(rng),
        "tag": rng.choice(TAGS),
    }


def edge_rows(include_nonstandard_floats: bool) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = [
        {"id": "", "timestamp": "2026-01-01T00:00:00Z", "value": 0.0, "tag": "empty-id"},
        {"id": "duplicate-id", "timestamp": "2026-01-01T00:00:00Z", "value": 1.0, "tag": "first"},
        {"id": "duplicate-id", "timestamp": "2026-01-01T00:00:01Z", "value": 2.0, "tag": "second"},
        {"id": "min-float", "timestamp": "1970-01-01T00:00:00Z", "value": -1.7976931348623157e308, "tag": "boundary"},
        {"id": "max-float", "timestamp": "9999-12-31T23:59:59.999999Z", "value": 1.7976931348623157e308, "tag": "boundary"},
        {"id": "negative-zero", "timestamp": "2038-01-19T03:14:07Z", "value": -0.0, "tag": "zero"},
        {"id": "csv-specials", "timestamp": "2026-01-01T00:00:00+02:00", "value": 12.34, "tag": 'comma, quote", newline\n'},
        {"id": "unicode", "timestamp": "2026-01-01T00:00:00.123456Z", "value": 3.141592653589793, "tag": "unicode-\u03c0-\U0001f680"},
        {"id": "invalid-timestamp", "timestamp": "not-a-date", "value": 10.0, "tag": "dirty"},
        {"id": "empty-timestamp", "timestamp": "", "value": 11.0, "tag": "dirty"},
        {"id": "empty-tag", "timestamp": "2026-01-01T00:00:00Z", "value": 12.0, "tag": ""},
    ]
    if include_nonstandard_floats:
        rows.extend(
            {"id": f"nonstandard-{idx}", "timestamp": "2026-01-01T00:00:00Z", "value": value, "tag": "dirty"}
            for idx, value in enumerate(NONSTANDARD_FLOATS, start=1)
        )
    return rows

# Writing to the actual csv file
def write_csv(path: Path, rows: list[dict[str, Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=FIELDNAMES)
        writer.writeheader()
        writer.writerows(rows)

# Individual dumps are quite slow here, will see if there isnt anything better for this 
def write_ndjson(path: Path, rows: list[dict[str, Any]], allow_nan: bool) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        for row in rows:
            handle.write(
                json.dumps(
                    row,
                    ensure_ascii=False,
                    allow_nan=allow_nan,
                    separators=(",", ":"),
                )
            )
            handle.write("\n")


def main() -> None:
    args = parse_args()
    # boundary checks 
    if args.rows < 0:
        raise SystemExit("--rows must be >= 0")
    if args.nonstandard_json_floats and not args.dirty:
        raise SystemExit("--nonstandard-json-floats only has an effect with --dirty")

    try:
        base = datetime.fromisoformat(args.start.replace("Z", "+00:00"))
    except ValueError as exc:
        raise SystemExit(f"--start must be ISO-8601 parseable: {args.start}") from exc
    if base.tzinfo is None:
        base = base.replace(tzinfo=timezone.utc)

    rng = random.Random(args.seed)
    rows = [random_row(rng, base) for _ in range(args.rows)]
    if args.dirty:
        rows = edge_rows(args.nonstandard_json_floats) + rows

    write_csv(args.csv, rows)
    write_ndjson(args.ndjson, rows, allow_nan=args.nonstandard_json_floats)

    print(f"wrote {len(rows)} rows to {args.csv} and {args.ndjson}")


if __name__ == "__main__":
    main()
