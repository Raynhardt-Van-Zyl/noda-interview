mod cli;
mod db;
mod input;
mod metrics;
mod model;
mod transform;

use anyhow::Result;
use clap::Parser;

use crate::{cli::Args, input::read_csv_records};

fn main() -> Result<()> {
    let args = Args::parse();
    let count = read_csv_records(&args.input, |_| Ok(()))?;

    println!("Read {count} records from {}", args.input.display());

    Ok(())
}
