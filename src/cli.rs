use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum InputFormat {
    Csv,
    Ndjson,
}

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Args {
    #[arg(long)]
    pub input: PathBuf,

    #[arg(long, value_enum)]
    pub format: InputFormat,

    #[arg(long)]
    pub db: PathBuf,

    #[arg(long, default_value_t = 1000)]
    pub batch_size: usize,
}
