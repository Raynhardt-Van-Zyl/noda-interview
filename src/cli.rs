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
}
