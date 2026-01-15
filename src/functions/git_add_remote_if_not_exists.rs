use crate::{ConvertStrToGitRemoteError, GitRemote};
use errgonomic::{handle, handle_iter, ErrVec};
use thiserror::Error;
use xshell::{cmd, Shell};

pub fn git_remote_exists(sh: &Shell, remote_template_url: &str) -> Result<bool, GitRemoteExistsError> {
    use GitRemoteExistsError::*;
    let output = handle!(cmd!(sh, "git remote -v").read(), ReadRemotesFailed);
    let results = output.lines().map(GitRemote::try_from);
    let remotes: Vec<GitRemote> = handle_iter!(results, ParseRemotesFailed);
    let remote_exists = remotes
        .iter()
        .any(|remote| remote.url == remote_template_url);
    Ok(remote_exists)
}

#[derive(Error, Debug)]
pub enum GitRemoteExistsError {
    #[error("failed to read git remotes")]
    ReadRemotesFailed { source: xshell::Error },
    #[error("failed to parse git remotes")]
    ParseRemotesFailed { source: ErrVec<ConvertStrToGitRemoteError> },
}
