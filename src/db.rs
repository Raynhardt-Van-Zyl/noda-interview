use anyhow::{Context, Result};
use rusqlite::{params, Connection};

use crate::model::CleanRecord;

pub fn open_connection(path: impl AsRef<std::path::Path>) -> Result<Connection> {
    let path = path.as_ref();
    Connection::open(path)
        .with_context(|| format!("failed to open SQLite database {}", path.display()))
}

pub fn insert_batch(connection: &mut Connection, records: &[CleanRecord]) -> Result<usize> {
    let transaction = connection
        .transaction()
        .context("failed to start SQLite transaction")?;
    let inserted = {
        let mut statement = transaction
            .prepare(
                "INSERT INTO metrics (id, timestamp, value, tag, positive)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
            )
            .context("failed to prepare metrics insert")?;
        let mut inserted = 0;

        for record in records {
            statement
                .execute(params![
                    record.id,
                    record.timestamp,
                    record.value,
                    record.tag,
                    i64::from(record.positive),
                ])
                .with_context(|| format!("failed to insert metrics row {}", record.id))?;
            inserted += 1;
        }

        inserted
    };

    transaction
        .commit()
        .context("failed to commit SQLite transaction")?;

    Ok(inserted)
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

        let inserted = insert_batch(&mut connection, &records).unwrap();
        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM metrics", [], |row| row.get(0))
            .unwrap();

        assert_eq!(inserted, 1);
        assert_eq!(count, 1);
    }
}
