use anyhow::{bail, Context, Result};
use chrono::DateTime;

use crate::model::{CleanRecord, RawRecord};

#[derive(Debug, Clone, PartialEq)]
pub enum TransformResult {
    Clean(CleanRecord),
    FilteredEmptyTag,
}

pub fn transform_record(record: RawRecord) -> Result<TransformResult> {
    if !record.value.is_finite() {
        bail!("value must be finite for id {}", record.id);
    }

    let tag = record.tag.trim().to_lowercase();
    if tag.is_empty() {
        return Ok(TransformResult::FilteredEmptyTag);
    }

    let timestamp = DateTime::parse_from_rfc3339(&record.timestamp)
        .with_context(|| format!("invalid timestamp for id {}", record.id))?
        .timestamp();

    Ok(TransformResult::Clean(CleanRecord {
        id: record.id,
        timestamp,
        value: record.value,
        tag,
        positive: record.value > 0.0,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw_record() -> RawRecord {
        RawRecord {
            id: "event-1".to_string(),
            timestamp: "2026-05-11T12:30:00Z".to_string(),
            value: 42.5,
            tag: " Prod ".to_string(),
        }
    }

    #[test]
    fn transforms_valid_record() {
        let result = transform_record(raw_record()).unwrap();

        assert_eq!(
            result,
            TransformResult::Clean(CleanRecord {
                id: "event-1".to_string(),
                timestamp: 1_778_502_600,
                value: 42.5,
                tag: "prod".to_string(),
                positive: true,
            })
        );
    }

    #[test]
    fn filters_empty_tags_after_trim() {
        let mut record = raw_record();
        record.tag = "  ".to_string();

        assert_eq!(
            transform_record(record).unwrap(),
            TransformResult::FilteredEmptyTag
        );
    }

    #[test]
    fn rejects_non_finite_values() {
        let mut record = raw_record();
        record.value = f64::INFINITY;

        assert!(transform_record(record).is_err());
    }
}
