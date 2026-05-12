use serde::Deserialize;

/// Record shape as it appears in CSV or NDJSON input.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct RawRecord {
    pub id: String,
    pub timestamp: String,
    pub value: f64,
    pub tag: String,
}

/// Validated record ready to be inserted into SQLite.
#[derive(Debug, Clone, PartialEq)]
pub struct CleanRecord {
    pub id: String,
    pub timestamp: i64,
    pub value: f64,
    pub tag: String,
    pub positive: bool,
}
