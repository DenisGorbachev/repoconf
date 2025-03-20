use crate::{GitRemote, Outcome};
use itertools::Itertools;
use xshell::{cmd, Shell};

pub fn git_remote_add_if_not_exists(sh: &Shell, remote_template_name: &str, remote_template_url: &str) -> Outcome {
    let remotes: Vec<GitRemote> = cmd!(sh, "git remote -v")
        .read()?
        .lines()
        .map(GitRemote::try_from)
        .try_collect()?;
    let remote_exists = remotes
        .iter()
        .any(|remote| remote.url == remote_template_url);
    if !remote_exists {
        cmd!(sh, "git remote add {remote_template_name} {remote_template_url}").run_echo()?;
    }
    Ok(())
}
