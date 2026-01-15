use crate::BranchNameStrategy;
use clap::builder::TypedValueParser;
use clap::error::ErrorKind;
use BranchNameStrategy::*;

#[derive(Clone, Debug)]
pub struct BranchNameStrategyValueParser;

impl TypedValueParser for BranchNameStrategyValueParser {
    type Value = BranchNameStrategy;

    fn parse_ref(&self, _cmd: &clap::Command, _arg: Option<&clap::Arg>, value: &std::ffi::OsStr) -> Result<Self::Value, clap::Error> {
        let val = match value.to_str() {
            Some(val) => val,
            None => {
                return Err(clap::Error::raw(ErrorKind::InvalidValue, "branch name argument must be valid UTF-8"));
            }
        };
        let strategy = if val == "-" { Auto } else { Exact(val.to_owned()) };
        Ok(strategy)
    }
}
