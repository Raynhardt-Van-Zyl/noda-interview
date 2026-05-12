use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use anyhow::{Context, Result};

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

/// Read CSV records with serde deserialization, one record at a time.
pub fn read_csv_records(
    path: impl AsRef<Path>,
    mut handle_record: impl FnMut(RawRecord) -> Result<()>,
) -> Result<usize> {
    let path = path.as_ref();
    let mut reader = csv::Reader::from_path(path)
        .with_context(|| format!("failed to open CSV input {}", path.display()))?;
    let mut count = 0;

    for record in reader.deserialize() {
        let record: RawRecord =
            record.with_context(|| format!("failed to parse CSV record in {}", path.display()))?;
        handle_record(record)?;
        count += 1;
    }

    Ok(count)
}

/// Read newline-delimited JSON records, one line at a time.
pub fn read_ndjson_records(
    path: impl AsRef<Path>,
    mut handle_record: impl FnMut(RawRecord) -> Result<()>,
) -> Result<usize> {
    let path = path.as_ref();
    let file = File::open(path)
        .with_context(|| format!("failed to open NDJSON input {}", path.display()))?;
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    let mut count = 0;

    loop {
        line.clear();
        let bytes_read = reader
            .read_line(&mut line)
            .with_context(|| format!("failed to read NDJSON input line in {}", path.display()))?;

        if bytes_read == 0 {
            break;
        }

        let record: RawRecord = serde_json::from_str(&line)
            .with_context(|| format!("failed to parse NDJSON record in {}", path.display()))?;
        handle_record(record)?;
        count += 1;
    }

    Ok(count)
}
