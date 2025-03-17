use crate::Outcome;
use clap::{value_parser, Parser};
use std::path::PathBuf;

#[derive(Parser, Clone, Debug)]
pub struct PrintCommand {
    #[arg(short, long, value_parser = value_parser!(PathBuf))]
    path: PathBuf,
}

impl PrintCommand {
    pub async fn run(self) -> Outcome {
        let Self {
            path,
        } = self;
        println!("{}", path.display());
        Ok(())
    }
}
