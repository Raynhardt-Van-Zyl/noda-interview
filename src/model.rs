//! Data contracts shared across parsing, transformation, insertion, and logging.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Record shape as it appears in CSV or NDJSON input.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RawRecord {
    /// Stable event identifier. This becomes the SQLite primary key.
    pub id: String,

    /// RFC 3339 timestamp string from the input file.
    pub timestamp: String,

    /// Numeric metric value. Transformation rejects NaN and infinite values.
    pub value: f64,

    /// Source tag. Transformation trims and lowercases this value.
    pub tag: String,
}

/// Validated record ready to be inserted into SQLite.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CleanRecord {
    /// Stable event identifier used as the SQLite primary key.
    pub id: String,

    /// Unix epoch seconds parsed from `RawRecord::timestamp`.
    pub timestamp: i64,

    /// Finite metric value copied from the raw record.
    pub value: f64,

    /// Normalized tag after trimming whitespace and lowercasing.
    pub tag: String,

    /// True when `value` is greater than zero.
    pub positive: bool,
}

/// Original input context for one physical input record.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RecordContext {
    /// One-based physical row number. CSV headers are not counted.
    pub row_number: usize,

    /// Input format label used in structured logs.
    pub format: &'static str,

    /// Original CSV fields or raw NDJSON line captured before parsing.
    pub raw: Value,
}

/// Clean record plus the source context needed for later diagnostics.
#[derive(Debug, Clone, PartialEq)]
pub struct PreparedRecord {
    /// Source context retained through batching.
    pub context: RecordContext,

    /// Validated record to insert into SQLite.
    pub record: CleanRecord,
}
