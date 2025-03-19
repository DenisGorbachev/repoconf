use crate::GitRemoteName;
use itertools::Itertools;
use xshell::{cmd, Shell};

pub trait GitRemoteNames {
    type Error;

    fn git_remote_names(&self) -> Result<impl Iterator<Item = GitRemoteName>, Self::Error>;
}

impl GitRemoteNames for Shell {
    type Error = xshell::Error;

    fn git_remote_names(&self) -> Result<impl Iterator<Item = GitRemoteName>, Self::Error> {
        let output = cmd!(self, "git remote").read()?;
        let iter = output
            .lines()
            .map(ToString::to_string)
            .collect_vec()
            .into_iter();
        Ok(iter)
    }
}
