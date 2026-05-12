mod cli;
mod db;
mod input;
mod metrics;
mod model;
mod transform;

use anyhow::{Result, bail};
use clap::Parser;

use crate::{
    cli::Args,
    db::{RunInserter, open_connection, with_run_inserter},
    input::read_records,
    metrics::RunMetrics,
    model::CleanRecord,
    transform::{TransformResult, transform_record},
};

fn main() -> Result<()> {
    let args = Args::parse();

    if args.batch_size == 0 {
        bail!("--batch-size must be greater than 0");
    }

    let mut connection = open_connection(&args.db)?;
    let mut metrics = RunMetrics::start();
    let mut batch = Vec::with_capacity(args.batch_size);

    with_run_inserter(&mut connection, |inserter| {
        // Transform one streamed record, append clean rows to a bounded
        // in-memory batch, and flush through one SQLite transaction.
        read_records(&args.input, args.format, |record| {
            metrics.total_records += 1;

            match transform_record(record) {
                Ok(TransformResult::Clean(record)) => {
                    batch.push(record);

                    if batch.len() >= args.batch_size {
                        flush_batch(inserter, &mut batch, &mut metrics);
                    }
                }
                Ok(TransformResult::FilteredEmptyTag) => {
                    metrics.filtered_empty_tags += 1;
                }
                Err(_) => {
                    metrics.failed_rows += 1;
                }
            }

            Ok(())
        })?;

        flush_batch(inserter, &mut batch, &mut metrics);

        Ok(())
    })?;

    println!("{}", metrics.summary());

    Ok(())
}

fn flush_batch(
    inserter: &mut RunInserter<'_>,
    batch: &mut Vec<CleanRecord>,
    metrics: &mut RunMetrics,
) {
    if batch.is_empty() {
        return;
    }

    let result = inserter.insert_batch(batch);
    metrics.successful_rows += result.inserted;
    metrics.failed_rows += result.failed;
    batch.clear();
}
