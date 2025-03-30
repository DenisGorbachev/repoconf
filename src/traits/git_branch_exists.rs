use std::io;
use xshell::{cmd, Shell};

pub trait GitLocalBranchExists {
    type Error;

    fn git_local_branch_exists(&self, branch_name: &str) -> Result<bool, Self::Error>;
}

impl GitLocalBranchExists for Shell {
    type Error = io::Error;

    fn git_local_branch_exists(&self, branch_name: &str) -> Result<bool, Self::Error> {
        let output = cmd!(self, "git show-ref --verify --quiet refs/heads/{branch_name}")
            .to_command()
            .status()?;
        Ok(output.success())
    }
}
