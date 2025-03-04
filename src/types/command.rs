use crate::Outcome;
use clap::Parser;
use std::io::{Read, Write};
use Command::*;

#[derive(Parser, Clone, Debug)]
pub enum Command {
    Print(PrintCommand),
    Create(CreateCommand),
}

impl Command {
    pub async fn run(self, stdin: &mut impl Read, stdout: &mut impl Write, stderr: &mut impl Write) -> Outcome {
        match self {
            Print(command) => command.run(stdin, stdout, stderr).await,
            Create(command) => command.run(stdin, stdout, stderr).await,
        }
    }
}

mod print_command;
pub use print_command::*;

mod create_command;
pub use create_command::*;
