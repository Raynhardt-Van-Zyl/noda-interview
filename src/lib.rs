//! Streaming CSV/NDJSON to SQLite ETL for embedding in Rust applications.
//!
//! `noda_interview` provides the reusable core behind the `noda-interview`
//! command line tool. It streams CSV or NDJSON records, validates and normalizes
//! each row, writes clean records to an existing SQLite `metrics` table, and can
//! emit structured JSONL diagnostics for failed or filtered rows.
//!
//! # Primary API
//!
//! Most applications should construct an [`EtlConfig`] and pass it to
//! [`run_etl`]:
//!
//! ```no_run
//! use noda_interview::{EtlConfig, cli::InputFormat, run_etl};
//!
//! # fn main() -> anyhow::Result<()> {
//! let config = EtlConfig::new("events.ndjson", InputFormat::Ndjson, "metrics.sqlite")
//!     .with_batch_size(1000)
//!     .with_log_file("events.jsonl");
//!
//! let metrics = run_etl(&config)?;
//! println!("{}", metrics.summary());
//! # Ok(())
//! # }
//! ```
//!
//! The SQLite database file and `metrics` table must already exist. The table
//! must contain the columns written by the loader:
//!
//! ```sql
//! CREATE TABLE metrics (
//!   id TEXT PRIMARY KEY,
//!   timestamp INTEGER NOT NULL,
//!   value REAL NOT NULL,
//!   tag TEXT NOT NULL,
//!   positive INTEGER NOT NULL
//! );
//! ```
//!
//! Additional SQLite columns are allowed when they are nullable or have a
//! default value.
//!
//! # Row-Level Failures
//!
//! Expected data-quality failures do not abort the run. Malformed input rows,
//! invalid timestamps, non-finite values, empty tags, and duplicate primary keys
//! are counted in [`metrics::RunMetrics`]. When `EtlConfig::log_file` is set,
//! each failed or filtered row is also written as one JSON Lines event with the
//! source row context.
//!
//! Fatal setup and operational problems still return an error from [`run_etl`].
//! Examples include a missing input file, missing SQLite database, incompatible
//! required schema, or a failure to flush the structured log.
//!
//! # Lower-Level Modules
//!
//! The top-level API is intentionally small, but lower-level modules are public
//! for integration tests and advanced embedding:
//!
//! - [`input`] streams CSV/NDJSON rows while preserving source context.
//! - [`transform`] validates and normalizes [`model::RawRecord`] values.
//! - [`db`] validates the SQLite table and inserts [`model::PreparedRecord`]
//!   batches.
//! - [`event_log`] writes structured diagnostics for rejected or skipped rows.

use std::path::PathBuf;

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
///
/// # Defaults
///
/// [`EtlConfig::new`] uses a batch size of `1000` and disables structured event
/// logging. Use [`with_batch_size`](Self::with_batch_size) and
/// [`with_log_file`](Self::with_log_file) to override those settings.
///
/// # Example
///
/// ```no_run
/// use noda_interview::{EtlConfig, cli::InputFormat};
///
/// let config = EtlConfig::new("input.csv", InputFormat::Csv, "metrics.sqlite")
///     .with_batch_size(500)
///     .with_log_file("events.jsonl");
///
/// assert_eq!(config.batch_size, 500);
/// ```
#[derive(Debug, Clone)]
pub struct EtlConfig {
    /// CSV or NDJSON file to read.
    ///
    /// The file is streamed; it is not loaded fully into memory.
    pub input: PathBuf,

    /// Parser to use for the input file.
    pub format: InputFormat,

    /// Existing SQLite database containing the target `metrics` table.
    ///
    /// Missing database files are treated as fatal setup errors.
    pub db: PathBuf,

    /// Number of clean records to collect before flushing to SQLite.
    ///
    /// Must be greater than zero.
    pub batch_size: usize,

    /// Optional JSON-lines file for failed and filtered rows.
    ///
    /// When set, parse, transform, filter, and database row failures are logged
    /// as structured events.
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
///
/// The returned [`RunMetrics`] has a frozen elapsed duration. Calling
/// [`RunMetrics::summary`] later will not include time spent after the ETL run
/// completed.
///
/// # Errors
///
/// Returns an error when setup or operational work fails, including:
///
/// - `batch_size == 0`
/// - input file open failures
/// - missing SQLite database file
/// - incompatible required `metrics` columns
/// - extra `NOT NULL` SQLite columns without defaults
/// - transaction or commit failures
/// - structured log write or flush failures
///
/// # Example
///
/// ```no_run
/// use noda_interview::{EtlConfig, cli::InputFormat, run_etl};
///
/// # fn main() -> anyhow::Result<()> {
/// let config = EtlConfig::new("events.csv", InputFormat::Csv, "metrics.sqlite");
/// let metrics = run_etl(&config)?;
///
/// println!("inserted {} rows", metrics.successful_rows);
/// # Ok(())
/// # }
/// ```
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
