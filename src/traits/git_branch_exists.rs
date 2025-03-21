use xshell::{cmd, Shell};

pub trait GitLocalBranchExists {
    type Error;

    fn git_local_branch_exists(&self, branch_name: &str) -> Result<bool, Self::Error>;
}

impl GitLocalBranchExists for Shell {
    type Error = xshell::Error;

    fn git_local_branch_exists(&self, branch_name: &str) -> Result<bool, Self::Error> {
        let output = cmd!(self, "git show-ref --verify --quiet refs/heads/{branch_name}").output()?;
        Ok(output.status.success())
    }
}
