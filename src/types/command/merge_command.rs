use crate::Outcome;
use clap::{value_parser, Parser};
use std::path::PathBuf;

#[derive(Parser, Clone, Debug)]
pub struct MergeCommand {
    #[arg(short, long, value_parser = value_parser!(PathBuf))]
    path: PathBuf,
}

impl MergeCommand {
    pub async fn run(self) -> Outcome {
        let Self {
            path,
        } = self;
        // TODO: git merge repoconf-template-rust-pre-public-lib/main --allow-unrelated-histories --no-commit --strategy subtree
        println!("{}", path.display());
        Ok(())
    }
}
