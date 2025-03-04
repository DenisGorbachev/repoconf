use std::io;
use std::path::PathBuf;
use std::process::Output;

pub trait EnsureBranch {
    type Output;
    fn ensure_branch(&self, name: &str) -> Self::Output;
}

impl EnsureBranch for PathBuf {
    type Output = io::Result<Output>;

    fn ensure_branch(&self, _name: &str) -> Self::Output {
        todo!()
    }
}
