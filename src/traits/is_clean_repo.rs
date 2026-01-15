use errgonomic::handle;
use thiserror::Error;
use xshell::{cmd, Shell};

pub trait IsCleanRepo {
    fn is_clean_repo(&self) -> Result<bool, IsCleanRepoError>;
}

impl IsCleanRepo for Shell {
    fn is_clean_repo(&self) -> Result<bool, IsCleanRepoError> {
        use IsCleanRepoError::*;
        let output = handle!(cmd!(self, "git status --porcelain").read(), ReadFailed);
        Ok(output.is_empty())
    }
}

#[derive(Error, Debug)]
pub enum IsCleanRepoError {
    #[error("failed to read git status")]
    ReadFailed { source: xshell::Error },
}
