use derive_more::{Error, From, Into};
use derive_new::new;
use std::fmt::{Display, Formatter};
use std::process::Output;

#[derive(new, Error, From, Into, Eq, PartialEq, Clone, Debug)]
pub struct UnexpectedOutput {
    output: Output,
}

impl UnexpectedOutput {}

/// Temporary implementation (TODO: remove it when proper error handling is implemented)
impl Display for UnexpectedOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:#?}")
    }
}
