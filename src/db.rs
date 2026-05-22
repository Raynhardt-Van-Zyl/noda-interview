use anyhow::{Context, Result, bail};
use rusqlite::{Connection, Error as SqliteError, OpenFlags, params};

use crate::model::{CleanRecord, PreparedRecord, RecordContext};

/// Insert outcome for one batch.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchInsertResult {
    /// Number of rows inserted successfully.
    pub inserted: usize,

    /// Number of expected row-level database failures.
    pub failed: usize,

    /// Database failures with source context retained for structured logging.
    pub failures: Vec<DatabaseRowFailure>,
}

/// Row-level database failure captured for structured logging.
#[derive(Debug, Clone, PartialEq)]
pub struct DatabaseRowFailure {
    /// Original source context for the failed row.
    pub context: RecordContext,

    /// Cleaned row that SQLite rejected.
    pub record: CleanRecord,

    /// Database error message, usually a constraint failure.
    pub reason: String,
}

/// Open the existing SQLite database used by the CLI run.
pub fn open_connection(path: impl AsRef<std::path::Path>) -> Result<Connection> {
    let path = path.as_ref();
    let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_WRITE)
        .with_context(|| format!("failed to open existing SQLite database {}", path.display()))?;

    validate_metrics_table(&connection)?;

    Ok(connection)
}

fn validate_metrics_table(connection: &Connection) -> Result<()> {
    let mut statement = connection
        .prepare("PRAGMA table_info(metrics)")
        .context("failed to inspect metrics table")?;
    let columns = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?.to_uppercase(),
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(5)?,
            ))
        })
        .context("failed to read metrics table schema")?
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("failed to collect metrics table schema")?;

    let expected = [
        ("id", "TEXT", 0, 1),
        ("timestamp", "INTEGER", 1, 0),
        ("value", "REAL", 1, 0),
        ("tag", "TEXT", 1, 0),
        ("positive", "INTEGER", 1, 0),
    ];

    if columns.len() != expected.len() {
        bail!("metrics table does not match expected schema");
    }

    for (actual, expected) in columns.iter().zip(expected) {
        let (name, column_type, not_null, primary_key) = actual;
        let (expected_name, expected_type, expected_not_null, expected_primary_key) = expected;
        if name != expected_name
            || column_type != expected_type
            || *not_null != expected_not_null
            || *primary_key != expected_primary_key
        {
            bail!("metrics table does not match expected schema");
        }
    }

    Ok(())
}

/// Insert one batch in a transaction.
///
/// Expected row-level constraint failures, such as duplicate primary keys, are
/// counted as failed rows. Operational database errors are returned to the CLI.
pub fn insert_batch(
    connection: &mut Connection,
    records: &[PreparedRecord],
) -> Result<BatchInsertResult> {
    let transaction = connection
        .transaction()
        .context("failed to start SQLite transaction")?;
    let result = {
        let mut statement = transaction
            .prepare(
                "INSERT INTO metrics (id, timestamp, value, tag, positive)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
            )
            .context("failed to prepare metrics insert")?;
        let mut inserted = 0;
        let mut failures = Vec::new();

        for record in records {
            let clean = &record.record;
            match statement.execute(params![
                clean.id,
                clean.timestamp,
                clean.value,
                clean.tag,
                i64::from(clean.positive),
            ]) {
                Ok(_) => inserted += 1,
                Err(error) if is_row_constraint_failure(&error) => {
                    failures.push(DatabaseRowFailure {
                        context: record.context.clone(),
                        record: clean.clone(),
                        reason: error.to_string(),
                    });
                }
                Err(error) => return Err(error).context("failed to insert metrics row"),
            }
        }

        BatchInsertResult {
            inserted,
            failed: failures.len(),
            failures,
        }
    };

    transaction
        .commit()
        .context("failed to commit SQLite transaction")?;

    Ok(result)
}

fn is_row_constraint_failure(error: &SqliteError) -> bool {
    matches!(error, SqliteError::SqliteFailure(failure, _) if failure.code == rusqlite::ErrorCode::ConstraintViolation)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn create_metrics_table(connection: &Connection) {
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

    fn prepared(row_number: usize, record: CleanRecord) -> PreparedRecord {
        PreparedRecord {
            context: RecordContext {
                row_number,
                format: "csv",
                raw: Value::Array(vec![Value::String(record.id.clone())]),
            },
            record,
        }
    }

    #[test]
    fn opens_existing_database_with_metrics_table() {
        let connection = Connection::open_in_memory().unwrap();
        create_metrics_table(&connection);

        validate_metrics_table(&connection).unwrap();
    }

    #[test]
    fn rejects_missing_metrics_table() {
        let connection = Connection::open_in_memory().unwrap();

        assert!(validate_metrics_table(&connection).is_err());
    }

    #[test]
    fn rejects_wrong_metrics_schema() {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute("CREATE TABLE metrics (id TEXT PRIMARY KEY)", [])
            .unwrap();

        assert!(validate_metrics_table(&connection).is_err());
    }

    #[test]
    fn inserts_clean_records() {
        let mut connection = Connection::open_in_memory().unwrap();
        create_metrics_table(&connection);
        let records = vec![prepared(
            1,
            CleanRecord {
                id: "event-1".to_string(),
                timestamp: 1_778_502_600,
                value: 42.5,
                tag: "prod".to_string(),
                positive: true,
            },
        )];

        let result = insert_batch(&mut connection, &records).unwrap();
        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM metrics", [], |row| row.get(0))
            .unwrap();

        assert_eq!(
            result,
            BatchInsertResult {
                inserted: 1,
                failed: 0,
                failures: Vec::new(),
            }
        );
        assert_eq!(count, 1);
    }

    #[test]
    fn keeps_inserting_after_duplicate_ids() {
        let mut connection = Connection::open_in_memory().unwrap();
        create_metrics_table(&connection);
        let records = vec![
            prepared(
                1,
                CleanRecord {
                    id: "event-1".to_string(),
                    timestamp: 1_778_502_600,
                    value: 42.5,
                    tag: "prod".to_string(),
                    positive: true,
                },
            ),
            prepared(
                2,
                CleanRecord {
                    id: "event-1".to_string(),
                    timestamp: 1_778_502_601,
                    value: -1.0,
                    tag: "prod".to_string(),
                    positive: false,
                },
            ),
            prepared(
                3,
                CleanRecord {
                    id: "event-2".to_string(),
                    timestamp: 1_778_502_602,
                    value: -1.0,
                    tag: "prod".to_string(),
                    positive: false,
                },
            ),
        ];

        let result = insert_batch(&mut connection, &records).unwrap();
        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM metrics", [], |row| row.get(0))
            .unwrap();

        assert_eq!(result.inserted, 2);
        assert_eq!(result.failed, 1);
        assert_eq!(result.failures.len(), 1);
        assert_eq!(result.failures[0].record.id, "event-1");
        assert_eq!(result.failures[0].context.row_number, 2);
        assert_eq!(count, 2);
    }
}
