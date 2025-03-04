use crate::Outcome;
use clap::{value_parser, Parser};
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser, Clone, Debug)]
pub struct PrintCommand {
    #[arg(short, long, value_parser = value_parser!(PathBuf))]
    path: PathBuf,
}

impl PrintCommand {
    pub async fn run(self, stdout: &mut impl Write, _stderr: &mut impl Write) -> Outcome {
        let Self {
            path,
        } = self;
        writeln!(stdout, "{}", path.display())?;
        Ok(())
    }
}
