use std::{fs, path::Path};

use noda_interview::{EtlConfig, cli::InputFormat, run_etl};
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
fn library_api_runs_etl_and_returns_metrics() {
    let temp_dir = tempdir().unwrap();
    let input_path = temp_dir.path().join("input.ndjson");
    let db_path = temp_dir.path().join("metrics.sqlite");
    let log_path = temp_dir.path().join("events.jsonl");
    create_metrics_db(&db_path);
    fs::write(
        &input_path,
        "{\"id\":\"ok\",\"timestamp\":\"2026-05-11T00:00:00Z\",\"value\":1.5,\"tag\":\"Prod\"}\n\
         {\"id\":\"empty\",\"timestamp\":\"2026-05-11T00:00:01Z\",\"value\":2.0,\"tag\":\"   \"}\n",
    )
    .unwrap();

    let config = EtlConfig::new(&input_path, InputFormat::Ndjson, &db_path)
        .with_batch_size(1)
        .with_log_file(&log_path);

    let metrics = run_etl(&config).unwrap();

    assert_eq!(metrics.total_records, 2);
    assert_eq!(metrics.successful_rows, 1);
    assert_eq!(metrics.failed_rows, 0);
    assert_eq!(metrics.filtered_empty_tags, 1);
    assert!(log_path.exists());

    let connection = Connection::open(db_path).unwrap();
    let tag: String = connection
        .query_row("SELECT tag FROM metrics WHERE id = 'ok'", [], |row| {
            row.get(0)
        })
        .unwrap();

    assert_eq!(tag, "prod");
}
