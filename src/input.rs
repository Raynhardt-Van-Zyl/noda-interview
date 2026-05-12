use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use anyhow::{Context, Result, anyhow};
use csv::ByteRecord;

use crate::{cli::InputFormat, model::RawRecord};

/// Stream records from the selected input format into the provided handler.
pub fn read_records(
    path: impl AsRef<Path>,
    format: InputFormat,
    handle_record: impl FnMut(RawRecord) -> Result<()>,
) -> Result<usize> {
    match format {
        InputFormat::Csv => read_csv_records(path, handle_record),
        InputFormat::Ndjson => read_ndjson_records(path, handle_record),
    }
}

/// Read CSV records one byte record at a time.
pub fn read_csv_records(
    path: impl AsRef<Path>,
    mut handle_record: impl FnMut(RawRecord) -> Result<()>,
) -> Result<usize> {
    let path = path.as_ref();
    let mut reader = csv::Reader::from_path(path)
        .with_context(|| format!("failed to open CSV input {}", path.display()))?;
    let headers = reader
        .byte_headers()
        .with_context(|| format!("failed to read CSV headers in {}", path.display()))?
        .clone();
    let columns = CsvColumns::from_headers(&headers, path)?;
    let mut record = ByteRecord::new();
    let mut count = 0;

    while reader
        .read_byte_record(&mut record)
        .with_context(|| format!("failed to read CSV record in {}", path.display()))?
    {
        let row_number = count + 2;
        let raw_record = columns.parse_record(&record).with_context(|| {
            format!(
                "failed to parse CSV record {row_number} in {}",
                path.display()
            )
        })?;
        handle_record(raw_record)?;
        count += 1;
    }

    Ok(count)
}

struct CsvColumns {
    id: usize,
    timestamp: usize,
    value: usize,
    tag: usize,
}

impl CsvColumns {
    fn from_headers(headers: &ByteRecord, path: &Path) -> Result<Self> {
        Ok(Self {
            id: required_column(headers, b"id", path)?,
            timestamp: required_column(headers, b"timestamp", path)?,
            value: required_column(headers, b"value", path)?,
            tag: required_column(headers, b"tag", path)?,
        })
    }

    fn parse_record(&self, record: &ByteRecord) -> Result<RawRecord> {
        Ok(RawRecord {
            id: field_string(record, self.id, "id")?,
            timestamp: field_string(record, self.timestamp, "timestamp")?,
            value: field_str(record, self.value, "value")?
                .parse()
                .context("invalid value field")?,
            tag: field_string(record, self.tag, "tag")?,
        })
    }
}

fn required_column(headers: &ByteRecord, name: &[u8], path: &Path) -> Result<usize> {
    headers
        .iter()
        .position(|header| header == name)
        .ok_or_else(|| {
            anyhow!(
                "CSV input {} is missing required header {}",
                path.display(),
                String::from_utf8_lossy(name)
            )
        })
}

fn field_string(record: &ByteRecord, index: usize, name: &str) -> Result<String> {
    Ok(field_str(record, index, name)?.to_owned())
}

fn field_str<'a>(record: &'a ByteRecord, index: usize, name: &str) -> Result<&'a str> {
    let field = record
        .get(index)
        .ok_or_else(|| anyhow!("missing {name} field"))?;
    std::str::from_utf8(field).with_context(|| format!("invalid UTF-8 in {name} field"))
}

/// Read newline-delimited JSON records, one line at a time.
pub fn read_ndjson_records(
    path: impl AsRef<Path>,
    mut handle_record: impl FnMut(RawRecord) -> Result<()>,
) -> Result<usize> {
    let path = path.as_ref();
    let file = File::open(path)
        .with_context(|| format!("failed to open NDJSON input {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut count = 0;

    for line in reader.lines() {
        let line = line
            .with_context(|| format!("failed to read NDJSON input line in {}", path.display()))?;
        let record: RawRecord = serde_json::from_str(&line)
            .with_context(|| format!("failed to parse NDJSON record in {}", path.display()))?;
        handle_record(record)?;
        count += 1;
    }

    Ok(count)
}
