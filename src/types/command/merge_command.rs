use crate::{git_refs, some_or_current_dir, BranchNameStrategy, GitLocalBranchExists, GitRemoteNames, IsCleanRepo, LocalBranchDoesNotExistError, Outcome, RepositoryNotCleanError};
use clap::{value_parser, Parser};
use itertools::Itertools;
use std::path::PathBuf;
use xshell::{cmd, Shell};

#[derive(Parser, Clone, Debug)]
pub struct MergeCommand {
    /// Child repository directory (defaults to current directory)
    #[arg(long, short, value_parser = value_parser!(PathBuf))]
    pub dir: Option<PathBuf>,

    /// Name of the local branch to merge onto
    ///
    /// If you pass "-", the command will determine the branch automatically: use "main" if exists, use "master" if exists.
    ///
    /// If the local branch doesn't exist, the command will exit with an error
    ///
    /// The command will switch to this branch before merging
    #[arg(long = "local-branch", short = 'l', default_value = "-")]
    pub local_branch_strategy: BranchNameStrategy,

    /// Name of the remote branch to merge from
    ///
    /// If you pass "-", the command will determine the branch automatically: use "main" if exists, use "master" if exists.
    ///
    /// Note that this is applied to all remotes
    #[arg(long = "remote-branch", short = 'r', default_value = "-")]
    pub remote_branch_strategy: BranchNameStrategy,
}

impl MergeCommand {
    pub async fn run(self) -> Outcome {
        let Self {
            dir,
            local_branch_strategy,
            remote_branch_strategy,
        } = self;

        let dir = some_or_current_dir(dir)?;
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

        let refs = git_refs(&sh)?;

        let local_branch_name = local_branch_strategy.to_branch_name("refs/heads", &refs)?;

        if !sh.git_local_branch_exists(&local_branch_name)? {
            return Err(LocalBranchDoesNotExistError::new(local_branch_name).into());
        }

        cmd!(sh, "git checkout {local_branch_name}").run_echo()?;

        cmd!(sh, "git remote update {remotes_slice...}").run_echo()?;

        for remote in remotes {
            let remote_branch_name = remote_branch_strategy.to_branch_name(&format!("refs/remotes/{remote}"), &refs)?;

            cmd!(sh, "git merge {remote}/{remote_branch_name}").run_echo()?;
        }

        Ok(())
    }
}
