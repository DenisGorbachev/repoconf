use crate::{GitRemoteName, IsCleanRepo, Outcome, RepositoryNotCleanError};
use clap::{value_parser, Parser};
use itertools::Itertools;
use std::path::{Path, PathBuf};
use stub_macro::stub;
use xshell::{cmd, Shell};

#[derive(Parser, Clone, Debug)]
pub struct MergeCommand {
    /// Name of the local branch to merge onto
    ///
    /// The command will switch to this branch before merging
    #[arg(long, short, default_value = "main")]
    local_branch_name: String,

    /// Name of the remote branch to merge from
    #[arg(long, short, default_value = "main")]
    remote_branch_name: String,

    /// Child repository directory
    #[arg(short, long, value_parser = value_parser!(PathBuf))]
    dir: PathBuf,
}

impl MergeCommand {
    pub async fn run(self) -> Outcome {
        let Self {
            local_branch_name,
            remote_branch_name,
            dir,
        } = self;
        let sh = Shell::new()?;

        if !sh.is_clean_repo()? {
            return Err(RepositoryNotCleanError::new().into());
        }

        cmd!(sh, "git checkout {local_branch_name}").run_echo()?;

        let remotes = git_remote_names(&dir)
            .filter(|name| name.starts_with("repoconf"))
            .collect_vec();
        let remotes_slice = remotes.as_slice();

        cmd!(sh, "git remote update {remotes_slice...}").run_echo()?;

        for remote in remotes {
            cmd!(sh, "git merge {remote}/{remote_branch_name}").run_echo()?;
        }

        Ok(())
    }
}

pub fn git_remote_names(_path: &Path) -> impl Iterator<Item = GitRemoteName> {
    stub!(impl dyn Iterator<Item=GitRemoteName>)
}
