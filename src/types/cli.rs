use crate::{Command, Outcome};
use clap::Parser;
use std::io::Write;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    // #[arg(short, long, value_parser = value_parser!(PathBuf))]
    // root: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

impl Cli {
    pub async fn run(self, stdout: &mut impl Write, stderr: &mut impl Write) -> Outcome {
        let Self {
            command,
        } = self;
        command.run(stdout, stderr).await
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
}
