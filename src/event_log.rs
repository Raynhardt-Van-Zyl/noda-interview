use std::{
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::json;

use crate::{
    cli::InputFormat,
    db::DatabaseRowFailure,
    input::InputRecordFailure,
    model::{CleanRecord, RawRecord, RecordContext},
};

/// Optional JSON-lines writer for rejected or skipped records.
pub struct EventLog {
    input_path: PathBuf,
    input_format: InputFormat,
    writer: Option<BufWriter<File>>,
}

impl EventLog {
    /// Create an optional event logger.
    ///
    /// When `path` is `None`, logging calls become no-ops while preserving the
    /// same call sites in the pipeline.
    pub fn open(
        path: Option<&Path>,
        input_path: impl AsRef<Path>,
        input_format: InputFormat,
    ) -> Result<Self> {
        let writer = match path {
            Some(path) => Some(BufWriter::new(File::create(path).with_context(|| {
                format!("failed to create log file {}", path.display())
            })?)),
            None => None,
        };

        Ok(Self {
            input_path: input_path.as_ref().to_path_buf(),
            input_format,
            writer,
        })
    }

    /// Log a row that could not be parsed into `RawRecord`.
    pub fn log_parse_failure(&mut self, failure: &InputRecordFailure) -> Result<()> {
        let context = failure
            .context
            .as_ref()
            .map(|context| self.record_context(context))
            .unwrap_or_else(|| self.base_context(None));

        self.write(json!({
            "event": "failed_row",
            "stage": "parse",
            "reason": failure.reason,
            "context": context,
            "entry": failure.context.as_ref().map(|context| &context.raw),
        }))
    }

    /// Log a row that parsed successfully but failed validation or normalization.
    pub fn log_transform_failure(
        &mut self,
        context: &RecordContext,
        record: &RawRecord,
        reason: &str,
    ) -> Result<()> {
        self.write(json!({
            "event": "failed_row",
            "stage": "transform",
            "reason": reason,
            "context": self.record_context(context),
            "entry": record,
        }))
    }

    /// Log a row skipped by the empty-tag business rule.
    pub fn log_filtered_empty_tag(
        &mut self,
        context: &RecordContext,
        record: &RawRecord,
    ) -> Result<()> {
        self.write(json!({
            "event": "filtered_empty_tag",
            "stage": "transform",
            "reason": "tag is empty after trimming whitespace",
            "context": self.record_context(context),
            "entry": record,
        }))
    }

    /// Log an expected row-level SQLite failure, such as a duplicate primary key.
    pub fn log_database_failure(&mut self, failure: &DatabaseRowFailure) -> Result<()> {
        self.write(json!({
            "event": "failed_row",
            "stage": "database",
            "reason": failure.reason,
            "context": self.record_context(&failure.context),
            "entry": LoggedCleanRecord(&failure.record),
        }))
    }

    /// Flush any buffered event log bytes and surface I/O errors to the caller.
    pub fn flush(&mut self) -> Result<()> {
        let Some(writer) = &mut self.writer else {
            return Ok(());
        };

        writer.flush().context("failed to flush structured log")
    }

    fn write(&mut self, event: serde_json::Value) -> Result<()> {
        let Some(writer) = &mut self.writer else {
            return Ok(());
        };

        serde_json::to_writer(&mut *writer, &event).context("failed to write structured log")?;
        writer
            .write_all(b"\n")
            .context("failed to write structured log newline")?;

        Ok(())
    }

    fn base_context(&self, row_number: Option<usize>) -> serde_json::Value {
        json!({
            "input_path": self.input_path,
            "format": format!("{:?}", self.input_format).to_lowercase(),
            "row_number": row_number,
        })
    }

    fn record_context(&self, context: &RecordContext) -> serde_json::Value {
        json!({
            "input_path": self.input_path,
            "format": context.format,
            "row_number": context.row_number,
            "raw": context.raw,
        })
    }
}

#[derive(Serialize)]
struct LoggedCleanRecord<'a>(&'a CleanRecord);
