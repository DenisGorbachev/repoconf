use crate::GitBranchName;
use clap::builder::{TypedValueParser, ValueParserFactory};
use clap::error::ErrorKind;
use derive_more::From;
use not_found_error::NotFoundError;
use BranchNameStrategy::*;

#[derive(From, Default, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Debug)]
pub enum BranchNameStrategy {
    #[default]
    Auto,
    Exact(String),
}

impl BranchNameStrategy {
    const AUTO_BRANCH_NAMES: [&'static str; 2] = ["main", "master"];

    pub fn to_branch_name(&self, prefix: &str, refs: &[String]) -> not_found_error::Result<GitBranchName> {
        match self {
            Auto => {
                for auto_branch_name in Self::AUTO_BRANCH_NAMES {
                    let auto_ref = format!("{prefix}/{auto_branch_name}");
                    if refs.iter().any(|r| r == &auto_ref) {
                        return Ok(auto_branch_name.into());
                    }
                }
                Err(NotFoundError::new())
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

#[derive(Clone, Debug)]
pub struct BranchNameStrategyValueParser;

impl TypedValueParser for BranchNameStrategyValueParser {
    type Value = BranchNameStrategy;

    fn parse_ref(&self, _cmd: &clap::Command, _arg: Option<&clap::Arg>, value: &std::ffi::OsStr) -> Result<Self::Value, clap::Error> {
        let val = value
            .to_str()
            .ok_or_else(|| clap::Error::raw(ErrorKind::InvalidValue, "Could not parse branch name argument"))?;
        let strategy = if val == "-" { Auto } else { Exact(val.to_owned()) };
        Ok(strategy)
    }
}
