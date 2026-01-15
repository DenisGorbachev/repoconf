use crate::GitRemoteName;
use errgonomic::handle;
use itertools::Itertools;
use thiserror::Error;
use xshell::{cmd, Shell};

pub trait GitRemoteNames {
    fn git_remote_names(&self) -> Result<impl Iterator<Item = GitRemoteName>, GitRemoteNamesError>;
}

impl GitRemoteNames for Shell {
    fn git_remote_names(&self) -> Result<impl Iterator<Item = GitRemoteName>, GitRemoteNamesError> {
        use GitRemoteNamesError::*;
        let output = handle!(cmd!(self, "git remote").read(), ReadFailed);
        let iter = output
            .lines()
            .map(ToString::to_string)
            .collect_vec()
            .into_iter();
        Ok(iter)
    }
}

#[derive(Error, Debug)]
pub enum GitRemoteNamesError {
    #[error("failed to read git remote names")]
    ReadFailed { source: xshell::Error },
}
