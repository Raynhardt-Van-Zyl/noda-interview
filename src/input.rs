use std::path::Path;

use anyhow::{Context, Result};

use crate::model::RawRecord;

pub fn read_csv_records(
    path: impl AsRef<Path>,
    mut handle_record: impl FnMut(RawRecord) -> Result<()>,
) -> Result<usize> {
    let path = path.as_ref();
    let mut reader = csv::Reader::from_path(path)
        .with_context(|| format!("failed to open CSV input {}", path.display()))?;
    let mut count = 0;

    for record in reader.deserialize() {
        let record: RawRecord = record
            .with_context(|| format!("failed to parse CSV record in {}", path.display()))?;
        handle_record(record)?;
        count += 1;
    }

    Ok(count)
}
