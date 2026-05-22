//! Streaming CSV/NDJSON to SQLite ETL.
//!
//! This crate provides the reusable core behind the `noda-interview` command
//! line tool. It streams CSV or NDJSON records, validates and normalizes each
//! row, writes clean records to an existing SQLite `metrics` table, and can emit
//! structured JSONL diagnostics for failed or filtered rows.
//!
//! The primary API for embedding the loader in another Rust codebase is
//! [`run_etl`]. Lower-level modules are public so callers can reuse individual
//! pieces, such as [`input`] readers, [`transform`] validation, or [`db`] batch
//! insertion.

use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

pub mod cli;
pub mod db;
pub mod event_log;
pub mod input;
pub mod metrics;
pub mod model;
pub mod transform;

use crate::{
    cli::InputFormat,
    db::{insert_batch, open_connection},
    event_log::EventLog,
    input::read_records,
    metrics::RunMetrics,
    model::PreparedRecord,
    transform::{TransformResult, transform_record},
};

/// Configuration for one ETL run.
///
/// This type is intentionally independent from Clap so library callers can
/// construct it directly without depending on CLI parsing.
#[derive(Debug, Clone)]
pub struct EtlConfig {
    /// CSV or NDJSON file to read.
    pub input: PathBuf,

    /// Parser to use for the input file.
    pub format: InputFormat,

    /// Existing SQLite database containing the target `metrics` table.
    pub db: PathBuf,

    /// Number of clean records to collect before flushing to SQLite.
    pub batch_size: usize,

    /// Optional JSON-lines file for failed and filtered rows.
    pub log_file: Option<PathBuf>,
}

impl EtlConfig {
    /// Build a configuration with the default batch size and no event log.
    pub fn new(input: impl Into<PathBuf>, format: InputFormat, db: impl Into<PathBuf>) -> Self {
        Self {
            input: input.into(),
            format,
            db: db.into(),
            batch_size: 1000,
            log_file: None,
        }
    }

    /// Override the clean-record batch size.
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }

    /// Enable structured JSONL diagnostics at the provided path.
    pub fn with_log_file(mut self, log_file: impl Into<PathBuf>) -> Self {
        self.log_file = Some(log_file.into());
        self
    }
}

/// Run the ETL pipeline and return aggregate runtime metrics.
///
/// The SQLite database file and `metrics` table must already exist. Expected
/// row-level failures, such as malformed records, invalid timestamps, empty
/// tags, non-finite values, or duplicate primary keys, are counted and logged
/// when `config.log_file` is set. Fatal setup and operational errors are
/// returned to the caller.
pub fn run_etl(config: &EtlConfig) -> Result<RunMetrics> {
    if config.batch_size == 0 {
        bail!("--batch-size must be greater than 0");
    }

    let mut connection = open_connection(&config.db)?;
    let mut metrics = RunMetrics::start();
    let mut event_log = EventLog::open(config.log_file.as_deref(), &config.input, config.format)?;
    let mut batch = Vec::with_capacity(config.batch_size);

    read_records(&config.input, config.format, |record| {
        metrics.total_records += 1;
        let input_record = match record {
            Ok(record) => record,
            Err(failure) => {
                metrics.failed_rows += 1;
                event_log.log_parse_failure(&failure)?;
                return Ok(());
            }
        };

        match transform_record(input_record.record.clone()) {
            Ok(TransformResult::Clean(clean_record)) => {
                batch.push(PreparedRecord {
                    context: input_record.context,
                    record: clean_record,
                });

                if batch.len() >= config.batch_size {
                    flush_batch(&mut connection, &mut batch, &mut metrics, &mut event_log)?;
                }
            }
            Ok(TransformResult::FilteredEmptyTag) => {
                metrics.filtered_empty_tags += 1;
                event_log.log_filtered_empty_tag(&input_record.context, &input_record.record)?;
            }
            Err(error) => {
                metrics.failed_rows += 1;
                event_log.log_transform_failure(
                    &input_record.context,
                    &input_record.record,
                    &error.to_string(),
                )?;
            }
        }

        Ok(())
    })?;

    flush_batch(&mut connection, &mut batch, &mut metrics, &mut event_log)?;
    event_log.flush()?;
    metrics.finish();

    Ok(metrics)
}

fn flush_batch(
    connection: &mut rusqlite::Connection,
    batch: &mut Vec<PreparedRecord>,
    metrics: &mut RunMetrics,
    event_log: &mut EventLog,
) -> Result<()> {
    if batch.is_empty() {
        return Ok(());
    }

    let result = insert_batch(connection, batch)?;
    metrics.successful_rows += result.inserted;
    metrics.failed_rows += result.failed;
    for failure in &result.failures {
        event_log.log_database_failure(failure)?;
    }
    batch.clear();

    Ok(())
}

impl From<&crate::cli::Args> for EtlConfig {
    fn from(args: &crate::cli::Args) -> Self {
        Self {
            input: args.input.clone(),
            format: args.format,
            db: args.db.clone(),
            batch_size: args.batch_size,
            log_file: args.log_file.clone(),
        }
    }
}

/// Return `true` when a path points to an existing regular file.
///
/// This helper is used by downstream examples and keeps doctests simple for
/// codebases that want to validate fixture paths before calling [`run_etl`].
pub fn is_existing_file(path: impl AsRef<Path>) -> bool {
    path.as_ref().is_file()
}
