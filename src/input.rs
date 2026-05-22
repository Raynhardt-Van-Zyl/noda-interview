//! Streaming readers for CSV and NDJSON inputs.
//!
//! The input layer is responsible for preserving source context. Every
//! successfully parsed row is returned as an [`InputRecord`], and every
//! row-level parse failure is returned as an [`InputRecordFailure`] when enough
//! source information is available to keep processing.
//!
//! This module does not validate timestamps, values, or tags. That work belongs
//! to [`crate::transform`].

use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use anyhow::{Context, Error, Result};
use serde_json::Value;

use crate::{
    cli::InputFormat,
    model::{RawRecord, RecordContext},
};

/// Parsed record with enough context to explain later validation failures.
#[derive(Debug, Clone)]
pub struct InputRecord {
    /// Source context captured before format-specific deserialization.
    pub context: RecordContext,

    /// Successfully deserialized raw record.
    pub record: RawRecord,
}

/// Parse failure with the original input entry where it was available.
#[derive(Debug, Clone)]
pub struct InputRecordFailure {
    /// Source row context. Some low-level read failures may not have raw data.
    pub context: Option<RecordContext>,

    /// Human-readable parse or read failure reason.
    pub reason: String,
}

/// Result passed to the pipeline for each physical input row.
pub type RecordReadResult = std::result::Result<InputRecord, InputRecordFailure>;

/// Stream records from the selected input format into the provided handler.
///
/// The handler receives a record read result for each physical input row. That
/// lets the caller count malformed rows, log the original entry, and continue
/// processing the rest of the file instead of aborting the whole ETL run on the
/// first parse error.
pub fn read_records(
    path: impl AsRef<Path>,
    format: InputFormat,
    handle_record: impl FnMut(RecordReadResult) -> Result<()>,
) -> Result<usize> {
    match format {
        InputFormat::Csv => read_csv_records(path, handle_record),
        InputFormat::Ndjson => read_ndjson_records(path, handle_record),
    }
}

/// Read CSV records with serde deserialization, one record at a time.
///
/// CSV input must include the header `id,timestamp,value,tag`. The callback is
/// invoked once per physical data row, excluding the header.
pub fn read_csv_records(
    path: impl AsRef<Path>,
    mut handle_record: impl FnMut(RecordReadResult) -> Result<()>,
) -> Result<usize> {
    let path = path.as_ref();
    let mut reader = csv::Reader::from_path(path)
        .with_context(|| format!("failed to open CSV input {}", path.display()))?;
    let headers = reader
        .headers()
        .with_context(|| format!("failed to read CSV headers in {}", path.display()))?
        .clone();
    let mut count = 0;

    for record in reader.records() {
        count += 1;
        let record = match record {
            Ok(record) => {
                let raw = Value::Array(record.iter().map(Value::from).collect());
                let context = RecordContext {
                    row_number: count,
                    format: "csv",
                    raw,
                };

                match record.deserialize::<RawRecord>(Some(&headers)) {
                    Ok(parsed) => Ok(InputRecord {
                        context,
                        record: parsed,
                    }),
                    Err(error) => Err(InputRecordFailure {
                        context: Some(context),
                        reason: Error::from(error)
                            .context(format!(
                                "failed to parse CSV record {} in {}",
                                count,
                                path.display()
                            ))
                            .to_string(),
                    }),
                }
            }
            Err(error) => Err(InputRecordFailure {
                context: Some(RecordContext {
                    row_number: count,
                    format: "csv",
                    raw: Value::Null,
                }),
                reason: Error::from(error)
                    .context(format!("failed to read CSV record in {}", path.display()))
                    .to_string(),
            }),
        };
        handle_record(record)?;
    }

    Ok(count)
}

/// Read newline-delimited JSON records, one line at a time.
///
/// NDJSON input must contain one JSON object per line. The raw line text is
/// retained in [`RecordContext`] so parse failures
/// can be logged with enough context for debugging.
pub fn read_ndjson_records(
    path: impl AsRef<Path>,
    mut handle_record: impl FnMut(RecordReadResult) -> Result<()>,
) -> Result<usize> {
    let path = path.as_ref();
    let file = File::open(path)
        .with_context(|| format!("failed to open NDJSON input {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut count = 0;

    for line in reader.lines() {
        count += 1;
        let record = line
            .with_context(|| format!("failed to read NDJSON input line in {}", path.display()))
            .map_err(|error| InputRecordFailure {
                context: Some(RecordContext {
                    row_number: count,
                    format: "ndjson",
                    raw: Value::Null,
                }),
                reason: error.to_string(),
            })
            .and_then(|line| {
                let context = RecordContext {
                    row_number: count,
                    format: "ndjson",
                    raw: Value::String(line.clone()),
                };

                match serde_json::from_str(&line) {
                    Ok(parsed) => Ok(InputRecord {
                        context,
                        record: parsed,
                    }),
                    Err(error) => Err(InputRecordFailure {
                        context: Some(context),
                        reason: Error::from(error)
                            .context(format!(
                                "failed to parse NDJSON record {} in {}",
                                count,
                                path.display()
                            ))
                            .to_string(),
                    }),
                }
            });
        handle_record(record)?;
    }

    Ok(count)
}
