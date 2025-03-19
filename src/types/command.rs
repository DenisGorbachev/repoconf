use crate::Outcome;
use clap::Parser;
use cli_util::command_enum;

command_enum!(
    #[derive(Parser, Clone, Debug)]
    pub enum Command {
        Create(CreateCommand),
        Merge(MergeCommand),
        Propagate(PropagateCommand),
    }
);

mod create_command;

pub use create_command::*;

mod merge_command;

pub use merge_command::*;

mod propagate_command;

pub use propagate_command::*;
