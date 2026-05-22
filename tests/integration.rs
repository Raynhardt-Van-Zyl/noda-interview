use std::{fs, path::Path};

use assert_cmd::Command;
use rusqlite::Connection;
use tempfile::tempdir;

fn create_metrics_db(path: &Path) {
    let connection = Connection::open(path).unwrap();
    connection
        .execute(
            "CREATE TABLE metrics (
                id TEXT PRIMARY KEY,
                timestamp INTEGER NOT NULL,
                value REAL NOT NULL,
                tag TEXT NOT NULL,
                positive INTEGER NOT NULL
            )",
            [],
        )
        .unwrap();
}

#[test]
fn processes_csv_into_sqlite_and_reports_metrics() {
    let temp_dir = tempdir().unwrap();
    let input_path = temp_dir.path().join("input.csv");
    let db_path = temp_dir.path().join("metrics.sqlite");
    create_metrics_db(&db_path);
    fs::write(
        &input_path,
        "id,timestamp,value,tag\n\
         ok,2026-05-11T00:00:00Z,1.5, Prod \n\
         empty,2026-05-11T00:00:01Z,2.0,   \n\
         bad-time,not-a-date,3.0,prod\n\
         dup,2026-05-11T00:00:03Z,4.0,prod\n\
         dup,2026-05-11T00:00:04Z,5.0,prod\n",
    )
    .unwrap();

    let output = Command::cargo_bin("noda-interview")
        .unwrap()
        .args([
            "--input",
            input_path.to_str().unwrap(),
            "--format",
            "csv",
            "--db",
            db_path.to_str().unwrap(),
            "--batch-size",
            "2",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();

    assert!(stdout.contains("Total records processed: 5"));
    assert!(stdout.contains("Successful rows written: 2"));
    assert!(stdout.contains("Failed rows: 2"));
    assert!(stdout.contains("Filtered empty tags: 1"));

    let connection = Connection::open(db_path).unwrap();
    let count: i64 = connection
        .query_row("SELECT COUNT(*) FROM metrics", [], |row| row.get(0))
        .unwrap();
    let tag: String = connection
        .query_row("SELECT tag FROM metrics WHERE id = 'ok'", [], |row| {
            row.get(0)
        })
        .unwrap();
    let positive: i64 = connection
        .query_row("SELECT positive FROM metrics WHERE id = 'ok'", [], |row| {
            row.get(0)
        })
        .unwrap();

    assert_eq!(count, 2);
    assert_eq!(tag, "prod");
    assert_eq!(positive, 1);
}

#[test]
fn writes_structured_json_log_for_failed_and_filtered_rows() {
    let temp_dir = tempdir().unwrap();
    let input_path = temp_dir.path().join("input.csv");
    let db_path = temp_dir.path().join("metrics.sqlite");
    let log_path = temp_dir.path().join("events.jsonl");
    create_metrics_db(&db_path);
    fs::write(
        &input_path,
        "id,timestamp,value,tag\n\
         empty,2026-05-11T00:00:01Z,2.0,   \n\
         bad-time,not-a-date,3.0,prod\n\
         dup,2026-05-11T00:00:03Z,4.0,prod\n\
         dup,2026-05-11T00:00:04Z,5.0,prod\n",
    )
    .unwrap();

    Command::cargo_bin("noda-interview")
        .unwrap()
        .args([
            "--input",
            input_path.to_str().unwrap(),
            "--format",
            "csv",
            "--db",
            db_path.to_str().unwrap(),
            "--batch-size",
            "2",
            "--log-file",
            log_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    let log = fs::read_to_string(log_path).unwrap();
    let events = log
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();

    assert_eq!(events.len(), 3);
    assert_eq!(events[0]["event"], "filtered_empty_tag");
    assert_eq!(events[0]["entry"]["id"], "empty");
    assert_eq!(events[0]["context"]["row_number"], 1);
    assert_eq!(events[1]["event"], "failed_row");
    assert_eq!(events[1]["stage"], "transform");
    assert_eq!(events[1]["entry"]["id"], "bad-time");
    assert_eq!(events[2]["event"], "failed_row");
    assert_eq!(events[2]["stage"], "database");
    assert_eq!(events[2]["entry"]["id"], "dup");
    assert_eq!(events[2]["context"]["row_number"], 4);
    assert_eq!(events[2]["context"]["raw"][0], "dup");
    assert_eq!(events[2]["context"]["raw"][1], "2026-05-11T00:00:04Z");
}

#[test]
fn writes_structured_json_log_with_raw_entry_for_parse_failures() {
    let temp_dir = tempdir().unwrap();
    let input_path = temp_dir.path().join("input.ndjson");
    let db_path = temp_dir.path().join("metrics.sqlite");
    let log_path = temp_dir.path().join("events.jsonl");
    create_metrics_db(&db_path);
    fs::write(
        &input_path,
        "{\"id\":\"ok\",\"timestamp\":\"2026-05-11T00:00:00Z\",\"value\":1.5,\"tag\":\"prod\"}\n\
         {not valid json}\n",
    )
    .unwrap();

    Command::cargo_bin("noda-interview")
        .unwrap()
        .args([
            "--input",
            input_path.to_str().unwrap(),
            "--format",
            "ndjson",
            "--db",
            db_path.to_str().unwrap(),
            "--log-file",
            log_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    let log = fs::read_to_string(log_path).unwrap();
    let events = log
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["event"], "failed_row");
    assert_eq!(events[0]["stage"], "parse");
    assert_eq!(events[0]["context"]["row_number"], 2);
    assert_eq!(events[0]["entry"], "{not valid json}");
}

#[test]
fn processes_ndjson_into_sqlite() {
    let temp_dir = tempdir().unwrap();
    let input_path = temp_dir.path().join("input.ndjson");
    let db_path = temp_dir.path().join("metrics.sqlite");
    create_metrics_db(&db_path);
    fs::write(
        &input_path,
        "{\"id\":\"event-1\",\"timestamp\":\"2026-05-11T00:00:00Z\",\"value\":-1.0,\"tag\":\"Beta\"}\n",
    )
    .unwrap();

    Command::cargo_bin("noda-interview")
        .unwrap()
        .args([
            "--input",
            input_path.to_str().unwrap(),
            "--format",
            "ndjson",
            "--db",
            db_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    let connection = Connection::open(db_path).unwrap();
    let row: (String, i64) = connection
        .query_row(
            "SELECT tag, positive FROM metrics WHERE id = 'event-1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();

    assert_eq!(row, ("beta".to_string(), 0));
}

#[test]
fn counts_malformed_csv_rows_as_failed_and_continues() {
    let temp_dir = tempdir().unwrap();
    let input_path = temp_dir.path().join("input.csv");
    let db_path = temp_dir.path().join("metrics.sqlite");
    create_metrics_db(&db_path);
    fs::write(
        &input_path,
        "id,timestamp,value,tag\n\
         ok-1,2026-05-11T00:00:00Z,1.5,prod\n\
         bad-value,2026-05-11T00:00:01Z,not-a-float,prod\n\
         ok-2,2026-05-11T00:00:02Z,-1.0, Beta \n",
    )
    .unwrap();

    let output = Command::cargo_bin("noda-interview")
        .unwrap()
        .args([
            "--input",
            input_path.to_str().unwrap(),
            "--format",
            "csv",
            "--db",
            db_path.to_str().unwrap(),
            "--batch-size",
            "1",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();

    assert!(stdout.contains("Total records processed: 3"));
    assert!(stdout.contains("Successful rows written: 2"));
    assert!(stdout.contains("Failed rows: 1"));

    let connection = Connection::open(db_path).unwrap();
    let count: i64 = connection
        .query_row("SELECT COUNT(*) FROM metrics", [], |row| row.get(0))
        .unwrap();

    assert_eq!(count, 2);
}

#[test]
fn counts_malformed_ndjson_rows_as_failed_and_continues() {
    let temp_dir = tempdir().unwrap();
    let input_path = temp_dir.path().join("input.ndjson");
    let db_path = temp_dir.path().join("metrics.sqlite");
    create_metrics_db(&db_path);
    fs::write(
        &input_path,
        "{\"id\":\"ok-1\",\"timestamp\":\"2026-05-11T00:00:00Z\",\"value\":1.5,\"tag\":\"prod\"}\n\
         {not valid json}\n\
         {\"id\":\"ok-2\",\"timestamp\":\"2026-05-11T00:00:02Z\",\"value\":-1.0,\"tag\":\" Beta \"}\n",
    )
    .unwrap();

    let output = Command::cargo_bin("noda-interview")
        .unwrap()
        .args([
            "--input",
            input_path.to_str().unwrap(),
            "--format",
            "ndjson",
            "--db",
            db_path.to_str().unwrap(),
            "--batch-size",
            "1",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();

    assert!(stdout.contains("Total records processed: 3"));
    assert!(stdout.contains("Successful rows written: 2"));
    assert!(stdout.contains("Failed rows: 1"));

    let connection = Connection::open(db_path).unwrap();
    let count: i64 = connection
        .query_row("SELECT COUNT(*) FROM metrics", [], |row| row.get(0))
        .unwrap();

    assert_eq!(count, 2);
}

#[test]
fn fails_when_database_file_does_not_exist() {
    let temp_dir = tempdir().unwrap();
    let input_path = temp_dir.path().join("input.csv");
    let db_path = temp_dir.path().join("missing.sqlite");
    fs::write(&input_path, "id,timestamp,value,tag\n").unwrap();

    Command::cargo_bin("noda-interview")
        .unwrap()
        .args([
            "--input",
            input_path.to_str().unwrap(),
            "--format",
            "csv",
            "--db",
            db_path.to_str().unwrap(),
        ])
        .assert()
        .failure();

    assert!(!db_path.exists());
}

#[test]
fn fails_when_metrics_table_is_missing() {
    let temp_dir = tempdir().unwrap();
    let input_path = temp_dir.path().join("input.csv");
    let db_path = temp_dir.path().join("metrics.sqlite");
    fs::write(&input_path, "id,timestamp,value,tag\n").unwrap();
    Connection::open(&db_path).unwrap();

    Command::cargo_bin("noda-interview")
        .unwrap()
        .args([
            "--input",
            input_path.to_str().unwrap(),
            "--format",
            "csv",
            "--db",
            db_path.to_str().unwrap(),
        ])
        .assert()
        .failure();
}

#[test]
fn fails_when_metrics_table_schema_is_wrong() {
    let temp_dir = tempdir().unwrap();
    let input_path = temp_dir.path().join("input.csv");
    let db_path = temp_dir.path().join("metrics.sqlite");
    fs::write(&input_path, "id,timestamp,value,tag\n").unwrap();
    let connection = Connection::open(&db_path).unwrap();
    connection
        .execute("CREATE TABLE metrics (id TEXT PRIMARY KEY)", [])
        .unwrap();

    Command::cargo_bin("noda-interview")
        .unwrap()
        .args([
            "--input",
            input_path.to_str().unwrap(),
            "--format",
            "csv",
            "--db",
            db_path.to_str().unwrap(),
        ])
        .assert()
        .failure();
}
