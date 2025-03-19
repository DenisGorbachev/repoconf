use crate::{GitRemoteNames, IsCleanRepo, Outcome, RepositoryNotCleanError};
use clap::{value_parser, Parser};
use itertools::Itertools;
use std::path::PathBuf;
use xshell::{cmd, Shell};

#[derive(Parser, Clone, Debug)]
pub struct MergeCommand {
    /// Name of the local branch to merge onto
    ///
    /// The command will switch to this branch before merging
    #[arg(long, short, default_value = "main")]
    pub local_branch_name: String,

    /// Name of the remote branch to merge from
    #[arg(long, short, default_value = "main")]
    pub remote_branch_name: String,

    /// Child repository directory
    #[arg(short, long, value_parser = value_parser!(PathBuf))]
    pub dir: PathBuf,
}

impl MergeCommand {
    pub async fn run(self) -> Outcome {
        let Self {
            local_branch_name,
            remote_branch_name,
            dir,
        } = self;
        let sh = Shell::new()?.with_current_dir(&dir);

        let remotes = sh
            .git_remote_names()?
            .filter(|name| name.starts_with("repoconf"))
            .collect_vec();

        // return early if this repository has no repoconf remotes
        // NOTE: [`PropagateCommand`] relies on this behavior
        if remotes.is_empty() {
            return Ok(());
        }

        let remotes_slice = remotes.as_slice();

        if !sh.is_clean_repo()? {
            return Err(RepositoryNotCleanError::new().into());
        }

        cmd!(sh, "git checkout {local_branch_name}").run_echo()?;

        cmd!(sh, "git remote update {remotes_slice...}").run_echo()?;

        for remote in remotes {
            cmd!(sh, "git merge {remote}/{remote_branch_name}").run_echo()?;
        }

        Ok(())
    }
}
