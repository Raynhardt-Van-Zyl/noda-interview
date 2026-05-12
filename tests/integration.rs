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
        "tag,value,timestamp,id\n\
         Prod ,1.5,2026-05-11T00:00:00Z,ok\n\
            ,2.0,2026-05-11T00:00:01Z,empty\n\
         prod,3.0,not-a-date,bad-time\n\
         prod,4.0,2026-05-11T00:00:03Z,dup\n\
         prod,5.0,2026-05-11T00:00:04Z,dup\n",
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
