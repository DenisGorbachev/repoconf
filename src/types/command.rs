use crate::Outcome;
use clap::Parser;
use std::io::Write;
use Command::*;

#[derive(Parser, Clone, Debug)]
pub enum Command {
    Print(PrintCommand),
}

impl Command {
    pub async fn run(self, stdout: &mut impl Write, stderr: &mut impl Write) -> Outcome {
        match self {
            Print(command) => command.run(stdout, stderr).await,
        }
    }
}

mod print_command;

pub use print_command::*;
