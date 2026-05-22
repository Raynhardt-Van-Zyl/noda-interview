use anyhow::Result;
use clap::Parser;
use noda_interview::{EtlConfig, cli::Args, run_etl};

fn main() -> Result<()> {
    let args = Args::parse();
    let config = EtlConfig::from(&args);
    let metrics = run_etl(&config)?;

    println!("{}", metrics.summary());

    Ok(())
}
