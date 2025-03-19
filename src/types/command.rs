use crate::Outcome;
use clap::Parser;
use cli_util::command_enum;

command_enum!(
    #[derive(Parser, Clone, Debug)]
    pub enum Command {
        Create(CreateCommand),
        Merge(MergeCommand),
    }
);

mod create_command;

pub use create_command::*;

mod merge_command;

pub use merge_command::*;
