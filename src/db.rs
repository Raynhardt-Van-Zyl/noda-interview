use anyhow::{Context, Result};
use rusqlite::{Connection, Statement, params};

use crate::model::CleanRecord;

/// Insert outcome for one batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatchInsertResult {
    pub inserted: usize,
    pub failed: usize,
}

/// Transaction-scoped inserter for one CLI run.
pub struct RunInserter<'tx> {
    statement: Statement<'tx>,
}

/// Open the existing SQLite database used by the CLI run.
pub fn open_connection(path: impl AsRef<std::path::Path>) -> Result<Connection> {
    let path = path.as_ref();
    Connection::open(path)
        .with_context(|| format!("failed to open SQLite database {}", path.display()))
}

/// Run insert work inside one transaction with one prepared statement.
pub fn with_run_inserter<R>(
    connection: &mut Connection,
    insert: impl FnOnce(&mut RunInserter<'_>) -> Result<R>,
) -> Result<R> {
    let transaction = connection
        .transaction()
        .context("failed to start SQLite transaction")?;
    let result = {
        let statement = transaction
            .prepare(
                "INSERT INTO metrics (id, timestamp, value, tag, positive)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
            )
            .context("failed to prepare metrics insert")?;
        let mut inserter = RunInserter { statement };
        insert(&mut inserter)?
    };

    transaction
        .commit()
        .context("failed to commit SQLite transaction")?;

    Ok(result)
}

impl RunInserter<'_> {
    /// Insert one bounded batch.
    ///
    /// Row-level SQLite errors, including duplicate primary keys, count as
    /// failed rows and insertion continues with the rest of the batch.
    pub fn insert_batch(&mut self, records: &[CleanRecord]) -> BatchInsertResult {
        let mut inserted = 0;
        let mut failed = 0;

        for record in records {
            match self.statement.execute(params![
                record.id,
                record.timestamp,
                record.value,
                record.tag,
                i64::from(record.positive),
            ]) {
                Ok(_) => inserted += 1,
                Err(_) => failed += 1,
            }
        }

        BatchInsertResult { inserted, failed }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn inserts_clean_records() {
        let mut connection = Connection::open_in_memory().unwrap();
        create_metrics_table(&connection);
        let records = vec![CleanRecord {
            id: "event-1".to_string(),
            timestamp: 1_778_502_600,
            value: 42.5,
            tag: "prod".to_string(),
            positive: true,
        }];

        let result = with_run_inserter(&mut connection, |inserter| {
            Ok(inserter.insert_batch(&records))
        })
        .unwrap();
        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM metrics", [], |row| row.get(0))
            .unwrap();

        assert_eq!(
            result,
            BatchInsertResult {
                inserted: 1,
                failed: 0
            }
        );
        assert_eq!(count, 1);
    }

    #[test]
    fn keeps_inserting_after_duplicate_ids() {
        let mut connection = Connection::open_in_memory().unwrap();
        create_metrics_table(&connection);
        let records = vec![
            CleanRecord {
                id: "event-1".to_string(),
                timestamp: 1_778_502_600,
                value: 42.5,
                tag: "prod".to_string(),
                positive: true,
            },
            CleanRecord {
                id: "event-1".to_string(),
                timestamp: 1_778_502_601,
                value: -1.0,
                tag: "prod".to_string(),
                positive: false,
            },
            CleanRecord {
                id: "event-2".to_string(),
                timestamp: 1_778_502_602,
                value: -1.0,
                tag: "prod".to_string(),
                positive: false,
            },
        ];

        let result = with_run_inserter(&mut connection, |inserter| {
            Ok(inserter.insert_batch(&records))
        })
        .unwrap();
        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM metrics", [], |row| row.get(0))
            .unwrap();

        assert_eq!(
            result,
            BatchInsertResult {
                inserted: 2,
                failed: 1
            }
        );
        assert_eq!(count, 2);
    }

    #[test]
    fn keeps_transaction_and_statement_across_batches() {
        let mut connection = Connection::open_in_memory().unwrap();
        create_metrics_table(&connection);

        let result = with_run_inserter(&mut connection, |inserter| {
            let first = inserter.insert_batch(&[CleanRecord {
                id: "event-1".to_string(),
                timestamp: 1_778_502_600,
                value: 42.5,
                tag: "prod".to_string(),
                positive: true,
            }]);
            let second = inserter.insert_batch(&[
                CleanRecord {
                    id: "event-1".to_string(),
                    timestamp: 1_778_502_601,
                    value: -1.0,
                    tag: "prod".to_string(),
                    positive: false,
                },
                CleanRecord {
                    id: "event-2".to_string(),
                    timestamp: 1_778_502_602,
                    value: -1.0,
                    tag: "prod".to_string(),
                    positive: false,
                },
            ]);

            Ok(BatchInsertResult {
                inserted: first.inserted + second.inserted,
                failed: first.failed + second.failed,
            })
        })
        .unwrap();
        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM metrics", [], |row| row.get(0))
            .unwrap();

        assert_eq!(
            result,
            BatchInsertResult {
                inserted: 2,
                failed: 1
            }
        );
        assert_eq!(count, 2);
    }
}
