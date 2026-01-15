use crate::{BranchNameStrategyValueParser, GitBranchName};
use clap::builder::ValueParserFactory;
use errgonomic::handle_opt;
use thiserror::Error;
use BranchNameStrategy::*;

#[derive(Default, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Debug)]
pub enum BranchNameStrategy {
    #[default]
    Auto,
    Exact(String),
}

impl BranchNameStrategy {
    const AUTO_BRANCH_NAMES: [&'static str; 2] = ["main", "master"];

    pub fn to_branch_name(&self, prefix: &str, refs: &[String]) -> Result<GitBranchName, BranchNameStrategyToBranchNameError> {
        use BranchNameStrategyToBranchNameError::*;
        match self {
            Auto => {
                let auto_branch = Self::AUTO_BRANCH_NAMES
                    .iter()
                    .find(|auto_branch_name| {
                        let auto_ref = format!("{prefix}/{auto_branch_name}");
                        refs.iter().any(|r| r == &auto_ref)
                    })
                    .copied();
                let auto_branch = handle_opt!(auto_branch, AutoBranchNotFound, prefix: prefix);
                Ok(auto_branch.to_string())
            }
            Exact(name) => Ok(name.to_owned()),
        }
    }
}

impl ValueParserFactory for BranchNameStrategy {
    type Parser = BranchNameStrategyValueParser;

    fn value_parser() -> Self::Parser {
        BranchNameStrategyValueParser
    }
}

#[derive(Error, Debug)]
pub enum BranchNameStrategyToBranchNameError {
    #[error("failed to find an auto branch under '{prefix}'")]
    AutoBranchNotFound { prefix: String },
}
