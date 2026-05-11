use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use anyhow::{Context, Result};

use crate::{cli::InputFormat, model::RawRecord};

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
