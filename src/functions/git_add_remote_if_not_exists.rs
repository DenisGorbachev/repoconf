use crate::{GitRemote, Outcome};
use itertools::Itertools;
use xshell::{cmd, Shell};

pub fn git_remote_exists(sh: &Shell, remote_template_url: &str) -> Outcome<bool> {
    let remotes: Vec<GitRemote> = cmd!(sh, "git remote -v")
        .read()?
        .lines()
        .map(GitRemote::try_from)
        .try_collect()?;
    let remote_exists = remotes
        .iter()
        .any(|remote| remote.url == remote_template_url);
    Ok(remote_exists)
}
