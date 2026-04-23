use clap::{Parser, value_parser};
use errgonomic::handle;
use std::path::PathBuf;
use std::process::ExitCode;
use thiserror::Error;
use tokio::fs::read_to_string;

#[derive(Parser, Clone, Debug)]
pub struct PrintCommand {
    #[arg(short, long, value_parser = value_parser!(PathBuf))]
    path: PathBuf,
}

impl PrintCommand {
    pub async fn run(self) -> Result<ExitCode, PrintCommandRunError> {
        use PrintCommandRunError::*;
        let Self {
            path,
        } = self;
        let contents = handle!(read_to_string(&path).await, ReadToStringFailed, path);
        println!("{contents}");
        Ok(ExitCode::SUCCESS)
    }
}

#[derive(Error, Debug)]
pub enum PrintCommandRunError {
    #[error("failed to read file at '{path}'")]
    ReadToStringFailed { source: std::io::Error, path: PathBuf },
}
