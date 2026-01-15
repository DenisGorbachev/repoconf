use errgonomic::handle;
use std::io;
use thiserror::Error;
use xshell::{cmd, Shell};

pub trait GitLocalBranchExists {
    fn git_local_branch_exists(&self, branch_name: &str) -> Result<bool, GitLocalBranchExistsError>;
}

impl GitLocalBranchExists for Shell {
    fn git_local_branch_exists(&self, branch_name: &str) -> Result<bool, GitLocalBranchExistsError> {
        use GitLocalBranchExistsError::*;
        let output = handle!(
            cmd!(self, "git show-ref --verify --quiet refs/heads/{branch_name}")
                .to_command()
                .status(),
            StatusFailed,
            branch_name: branch_name
        );
        Ok(output.success())
    }
}

#[derive(Error, Debug)]
pub enum GitLocalBranchExistsError {
    #[error("failed to check local branch '{branch_name}'")]
    StatusFailed { source: io::Error, branch_name: String },
}
