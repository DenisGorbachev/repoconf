use crate::Outcome;
use clap::Parser;
use Command::*;

#[derive(Parser, Clone, Debug)]
pub enum Command {
    Print(PrintCommand),
    Create(CreateCommand),
    Merge(MergeCommand),
}

impl Command {
    pub async fn run(self) -> Outcome {
        match self {
            Print(command) => command.run().await,
            Create(command) => command.run().await,
            Merge(command) => command.run().await,
        }
    }
}

mod print_command;

pub use print_command::*;

mod create_command;

pub use create_command::*;

mod merge_command;

pub use merge_command::*;
