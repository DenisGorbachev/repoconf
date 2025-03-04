use crate::Strip;
use derive_getters::Getters;
use derive_more::From;
use std::path::Path;

#[derive(Getters, From, Ord, PartialOrd, Eq, PartialEq, Default, Hash, Clone, Debug)]
pub struct WithEmptyFirstCommit<T: AsRef<Path>> {
    value: T,
}

#[allow(dead_code)]
pub struct WithoutEmptyFirstCommit<T: AsRef<Path>> {
    value: T,
}

pub trait WithEmptyFirstCommitLike<T: AsRef<Path>> {
    fn to_own(self) -> WithEmptyFirstCommit<T>;
    fn to_ref(&self) -> &WithEmptyFirstCommit<T>;
}

impl<T: AsRef<Path>> WithEmptyFirstCommit<T> {
    pub fn new(_value: T) -> Result<Self, WithoutEmptyFirstCommit<T>> {
        todo!()
    }
}

impl<T: AsRef<Path> + Strip> Strip for WithEmptyFirstCommit<T> {
    type Output = <T as Strip>::Output;

    fn strip(self) -> Self::Output {
        self.value.strip()
    }
}
