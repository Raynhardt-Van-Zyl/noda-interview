use std::path::PathBuf;

use clap::{Parser, ValueEnum};

/// Supported streaming input formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum InputFormat {
    Csv,
    Ndjson,
}

/// Command-line options for one ETL run.
#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Args {
    /// CSV or NDJSON file to read.
    #[arg(long)]
    pub input: PathBuf,

    /// Parser to use for the input file.
    #[arg(long, value_enum)]
    pub format: InputFormat,

    /// SQLite database containing the target `metrics` table.
    #[arg(long)]
    pub db: PathBuf,

    /// Number of clean records to collect before flushing to SQLite.
    #[arg(long, default_value_t = 1000)]
    pub batch_size: usize,

    /// Optional JSON-lines file for failed and filtered input records.
    #[arg(long)]
    pub log_file: Option<PathBuf>,
}
