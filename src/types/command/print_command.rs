use crate::Outcome;
use clap::{value_parser, Parser};
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Parser, Clone, Debug)]
pub struct PrintCommand {
    #[arg(short, long, value_parser = value_parser!(PathBuf))]
    path: PathBuf,
}

impl PrintCommand {
    pub async fn run(self, _stdin: &mut impl Read, stdout: &mut impl Write, _stderr: &mut impl Write) -> Outcome {
        let Self {
            path,
        } = self;
        writeln!(stdout, "{}", path.display())?;
        Ok(())
    }
}
