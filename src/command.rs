use errgonomic::map_err;
use thiserror::Error;
use Subcommand::*;

#[derive(clap::Parser, Debug)]
#[command(author, version, about, propagate_version = true)]
pub struct Command {
    #[command(subcommand)]
    subcommand: Subcommand,
}

#[derive(clap::Subcommand, Clone, Debug)]
pub enum Subcommand {
    Add(AddCommand),
    Create(CreateCommand),
    Init(InitCommand),
    Merge(MergeCommand),
    Propagate(PropagateCommand),
}

impl Command {
    pub async fn run(self) -> Result<(), CommandRunError> {
        use CommandRunError::*;
        let Self {
            subcommand,
        } = self;
        match subcommand {
            Add(command) => map_err!(command.run().await, AddCommandRunFailed),
            Create(command) => map_err!(command.run().await, CreateCommandRunFailed),
            Init(command) => map_err!(command.run().await, InitCommandRunFailed),
            Merge(command) => map_err!(command.run().await, MergeCommandRunFailed),
            Propagate(command) => map_err!(command.run().await, PropagateCommandRunFailed),
        }
    }
}

#[derive(Error, Debug)]
pub enum CommandRunError {
    #[error("failed to run add command")]
    AddCommandRunFailed { source: AddCommandRunError },
    #[error("failed to run create command")]
    CreateCommandRunFailed { source: CreateCommandRunError },
    #[error("failed to run init command")]
    InitCommandRunFailed { source: InitCommandRunError },
    #[error("failed to run merge command")]
    MergeCommandRunFailed { source: MergeCommandRunError },
    #[error("failed to run propagate command")]
    PropagateCommandRunFailed { source: PropagateCommandRunError },
}

mod add_command;
pub use add_command::*;
mod create_command;
pub use create_command::*;
mod init_command;
pub use init_command::*;
mod merge_command;
pub use merge_command::*;
mod propagate_command;
pub use propagate_command::*;
