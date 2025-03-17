use crate::{Command, Outcome};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    // #[arg(short, long, value_parser = value_parser!(PathBuf))]
    // root: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

impl Cli {
    pub async fn run(self) -> Outcome {
        let Self {
            command,
        } = self;
        command.run().await
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
}
