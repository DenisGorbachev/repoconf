use crate::{git_refs, unwrap_or_current_dir, BranchNameStrategy, BranchNameStrategyToBranchNameError, GitLocalBranchExists, GitLocalBranchExistsError, GitRefsError, GitRemoteNames, GitRemoteNamesError, IsCleanRepo, IsCleanRepoError, UnwrapOrCurrentDirError};
use clap::{value_parser, Parser};
use errgonomic::{handle, handle_bool};
use itertools::Itertools;
use std::path::PathBuf;
use thiserror::Error;
use xshell::{cmd, Shell};

#[derive(Parser, Default, Clone, Debug)]
pub struct MergeCommand {
    /// Child repository directory (defaults to current directory)
    #[arg(long, short, value_parser = value_parser!(PathBuf))]
    pub dir: Option<PathBuf>,

    /// Run the command even if the repository has uncommitted changes
    #[arg(long)]
    pub allow_dirty: bool,

    #[arg(long)]
    pub allow_unrelated_histories: bool,

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
    pub async fn run(self) -> Result<(), MergeCommandRunError> {
        use MergeCommandRunError::*;
        let Self {
            dir,
            allow_dirty,
            allow_unrelated_histories,
            local_branch_strategy,
            remote_branch_strategy,
        } = self;

        let dir = handle!(unwrap_or_current_dir(dir), UnwrapOrCurrentDirFailed);
        let sh_dir = handle!(Shell::new(), ShellNewFailed).with_current_dir(&dir);

        let remotes = handle!(sh_dir.git_remote_names(), GitRemoteNamesFailed)
            .filter(|name| name.starts_with("repoconf"))
            .collect_vec();

        // NOTE: [`PropagateCommand`] relies on this behavior
        if remotes.is_empty() {
            return Ok(());
        }

        let is_clean = handle!(sh_dir.is_clean_repo(), IsCleanRepoFailed);
        handle_bool!(!allow_dirty && !is_clean, RepositoryNotClean, dir);

        let refs = handle!(git_refs(&sh_dir), GitRefsFailed);

        let local_branch_name = handle!(
            local_branch_strategy.to_branch_name("refs/heads", &refs),
            LocalBranchNameResolveFailed,
            prefix: "refs/heads",
            strategy: local_branch_strategy
        );

        let local_branch_exists = handle!(
            sh_dir.git_local_branch_exists(&local_branch_name),
            GitLocalBranchExistsFailed,
            branch_name: local_branch_name
        );
        handle_bool!(!local_branch_exists, LocalBranchDoesNotExist, branch_name: local_branch_name);

        handle!(
            cmd!(sh_dir, "git checkout {local_branch_name}").run_echo(),
            GitCheckoutFailed,
            branch_name: local_branch_name
        );

        let remotes_slice = remotes.as_slice();
        handle!(cmd!(sh_dir, "git remote update {remotes_slice...}").run_echo(), GitRemoteUpdateFailed, remotes);

        handle!(Self::merge_remotes(&sh_dir, remotes, &remote_branch_strategy, &refs, allow_unrelated_histories), MergeRemotesFailed);

        handle!(cmd!(sh_dir, "git push").run_echo(), GitPushFailed);

        Ok(())
    }

    fn merge_remotes(sh_dir: &Shell, remotes: Vec<String>, remote_branch_strategy: &BranchNameStrategy, refs: &[String], allow_unrelated_histories: bool) -> Result<(), MergeCommandMergeRemotesError> {
        use MergeCommandMergeRemotesError::*;
        remotes.into_iter().try_for_each(|remote| {
            let remote_name = remote;
            handle!(
                Self::merge_remote(sh_dir, remote_branch_strategy, refs, allow_unrelated_histories, &remote_name),
                MergeRemoteFailed,
                remote: remote_name
            );
            Ok(())
        })
    }

    fn merge_remote(sh_dir: &Shell, remote_branch_strategy: &BranchNameStrategy, refs: &[String], allow_unrelated_histories: bool, remote: &str) -> Result<(), MergeCommandMergeRemoteError> {
        use MergeCommandMergeRemoteError::*;
        let remote = remote.to_string();
        let remote_prefix = format!("refs/remotes/{remote}");
        let remote_branch_name = handle!(
            remote_branch_strategy.to_branch_name(&remote_prefix, refs),
            RemoteBranchNameResolveFailed,
            prefix: remote_prefix,
            remote
        );

        let flags = if allow_unrelated_histories {
            vec!["--allow-unrelated-histories", "--no-commit"]
        } else {
            Vec::new()
        };

        handle!(cmd!(sh_dir, "git merge {remote}/{remote_branch_name} {flags...}").run_echo(), GitMergeFailed, remote, remote_branch_name);

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum MergeCommandRunError {
    #[error("failed to resolve the target directory")]
    UnwrapOrCurrentDirFailed { source: UnwrapOrCurrentDirError },
    #[error("failed to create a shell instance")]
    ShellNewFailed { source: xshell::Error },
    #[error("failed to read git remote names")]
    GitRemoteNamesFailed { source: GitRemoteNamesError },
    #[error("failed to check repository status")]
    IsCleanRepoFailed { source: IsCleanRepoError },
    #[error("repository '{dir}' has uncommitted changes")]
    RepositoryNotClean { dir: PathBuf },
    #[error("failed to read git refs")]
    GitRefsFailed { source: GitRefsError },
    #[error("failed to resolve local branch name for prefix '{prefix}'")]
    LocalBranchNameResolveFailed { source: BranchNameStrategyToBranchNameError, prefix: String, strategy: BranchNameStrategy },
    #[error("failed to check whether local branch '{branch_name}' exists")]
    GitLocalBranchExistsFailed { source: GitLocalBranchExistsError, branch_name: String },
    #[error("local branch '{branch_name}' does not exist")]
    LocalBranchDoesNotExist { branch_name: String },
    #[error("failed to check out local branch '{branch_name}'")]
    GitCheckoutFailed { source: xshell::Error, branch_name: String },
    #[error("failed to update repoconf remotes")]
    GitRemoteUpdateFailed { source: xshell::Error, remotes: Vec<String> },
    #[error("failed to merge remotes")]
    MergeRemotesFailed { source: MergeCommandMergeRemotesError },
    #[error("failed to push merged changes")]
    GitPushFailed { source: xshell::Error },
}

#[derive(Error, Debug)]
pub enum MergeCommandMergeRemotesError {
    #[error("failed to merge from remote '{remote}'")]
    MergeRemoteFailed { source: MergeCommandMergeRemoteError, remote: String },
}

#[derive(Error, Debug)]
pub enum MergeCommandMergeRemoteError {
    #[error("failed to resolve remote branch name for '{remote}' with prefix '{prefix}'")]
    RemoteBranchNameResolveFailed { source: BranchNameStrategyToBranchNameError, prefix: String, remote: String },
    #[error("failed to merge from '{remote}/{remote_branch_name}'")]
    GitMergeFailed { source: xshell::Error, remote: String, remote_branch_name: String },
}
