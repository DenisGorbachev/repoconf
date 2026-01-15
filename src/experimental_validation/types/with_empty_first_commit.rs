use crate::Strip;
use derive_getters::Getters;
use derive_more::From;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Getters, From, Ord, PartialOrd, Eq, PartialEq, Default, Hash, Clone, Debug)]
pub struct WithEmptyFirstCommit<T: AsRef<Path>> {
    value: T,
}

pub trait WithEmptyFirstCommitLike<T: AsRef<Path>> {
    fn to_own(self) -> WithEmptyFirstCommit<T>;
    fn to_ref(&self) -> &WithEmptyFirstCommit<T>;
}

impl<T: AsRef<Path>> WithEmptyFirstCommit<T> {
    pub fn new(value: T) -> Result<Self, WithEmptyFirstCommitNewError> {
        use WithEmptyFirstCommitNewError::*;
        let _ = EmptyFirstCommitMissing {
            path: value.as_ref().to_path_buf(),
        };
        todo!()
    }
}

#[derive(Error, Debug)]
pub enum WithEmptyFirstCommitNewError {
    #[error("path '{path}' does not have an empty first commit")]
    EmptyFirstCommitMissing { path: PathBuf },
}

impl<T: AsRef<Path> + Strip> Strip for WithEmptyFirstCommit<T> {
    type Output = <T as Strip>::Output;

    fn strip(self) -> Self::Output {
        self.value.strip()
    }
}
