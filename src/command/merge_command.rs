use crate::{BranchNameStrategy, BranchNameStrategyToBranchNameError, GitBranchName, GitLocalBranchExists, GitLocalBranchExistsError, GitRefsError, GitRemoteNames, GitRemoteNamesError, IsCleanRepo, IsCleanRepoError, UnwrapOrCurrentDirError, git_refs, unwrap_or_current_dir};
use clap::{Parser, value_parser};
use errgonomic::{handle, handle_bool, handle_opt_take};
use itertools::Itertools;
use std::path::PathBuf;
use std::process::ExitCode;
use thiserror::Error;
use xshell::{Shell, cmd};

#[derive(Parser, Default, Clone, Debug)]
#[command(flatten_help = true, about = "Merge templates, automatically completing resolved pending merges")]
pub struct MergeCommand {
    /// Child repository directory (defaults to current directory)
    #[arg(long, short, value_parser = value_parser!(PathBuf))]
    pub dir: Option<PathBuf>,

    /// Run the command even if the repository has uncommitted changes
    #[arg(long)]
    pub allow_dirty: bool,

    /// Skip ordinary uncommitted changes after finishing any pending merge
    #[arg(long, conflicts_with = "allow_dirty")]
    pub skip_dirty: bool,

    #[arg(long)]
    pub allow_unrelated_histories: bool,

    /// Do not push merged changes after merging
    #[arg(long, env = "REPOCONF_NO_PUSH")]
    pub no_push: bool,

    #[arg(long, env = "REPOCONF_NO_PULL", help = "Do not pull the local branch's upstream before merging")]
    pub no_pull: bool,

    /// Do not update template remotes before merging
    #[arg(long)]
    pub no_remote_update: bool,

    /// Do not run the post-merge hook after merging
    #[arg(long)]
    pub skip_post_merge: bool,

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
    pub async fn run(self) -> Result<ExitCode, MergeCommandRunError> {
        use MergeCommandRunError::*;
        let Self {
            dir,
            allow_dirty,
            skip_dirty,
            allow_unrelated_histories,
            no_push,
            no_pull,
            no_remote_update,
            skip_post_merge,
            local_branch_strategy,
            remote_branch_strategy,
        } = self;

        let dir = handle!(unwrap_or_current_dir(dir), UnwrapOrCurrentDirFailed);
        let sh_dir = handle!(Shell::new(), ShellNewFailed).with_current_dir(&dir);

        let remotes = handle!(sh_dir.git_remote_names(), GitRemoteNamesFailed)
            .filter(|name| name.starts_with("repoconf-"))
            .collect_vec();

        // NOTE: [`PropagateCommand`] relies on this behavior
        if remotes.is_empty() {
            return Ok(ExitCode::SUCCESS);
        }

        let continued_branch_name = handle!(Self::finish_pending_merge(&sh_dir, &local_branch_strategy), FinishPendingMergeFailed, dir);
        let is_clean = handle!(sh_dir.is_clean_repo(), IsCleanRepoFailed, dir);
        if handle!(Self::should_skip_dirty(is_clean, allow_dirty, skip_dirty), ShouldSkipDirtyFailed, dir, is_clean, allow_dirty, skip_dirty) {
            eprintln!("[SKIP] repository '{}' has uncommitted changes", dir.display());
            return Ok(ExitCode::SUCCESS);
        }

        let local_branch_name = match continued_branch_name {
            Some(branch_name) => branch_name,
            None => handle!(Self::resolve_local_branch(&sh_dir, &local_branch_strategy), ResolveLocalBranchFailed, dir),
        };

        handle!(
            cmd!(sh_dir, "git checkout {local_branch_name}").run_echo(),
            GitCheckoutFailed,
            branch_name: local_branch_name
        );

        if !no_pull {
            handle!(Self::pull(&sh_dir, &local_branch_name), PullFailed, dir, branch_name: local_branch_name);
        }

        if !no_remote_update {
            let remotes_slice = remotes.as_slice();
            handle!(cmd!(sh_dir, "git remote update {remotes_slice...}").run_echo(), GitRemoteUpdateFailed, remotes);
        }

        let refs = handle!(git_refs(&sh_dir), GitRemoteRefsFailed);
        handle!(Self::merge_remotes(&sh_dir, remotes, &remote_branch_strategy, &refs, allow_unrelated_histories), MergeRemotesFailed);

        if !skip_post_merge {
            let post_merge_path = sh_dir.current_dir().join(".repoconf/hooks/post-merge.sh");
            handle!(Self::run_hook(&sh_dir, post_merge_path), RunHookFailed);
        }

        if !no_push {
            handle!(cmd!(sh_dir, "git push").run_echo(), GitPushFailed);
        }

        Ok(ExitCode::SUCCESS)
    }

    fn should_skip_dirty(is_clean: bool, allow_dirty: bool, skip_dirty: bool) -> Result<bool, MergeCommandShouldSkipDirtyError> {
        use MergeCommandShouldSkipDirtyError::*;
        handle_bool!(!is_clean && !allow_dirty && !skip_dirty, RepositoryNotClean);
        Ok(skip_dirty && !is_clean)
    }

    fn resolve_local_branch(sh_dir: &Shell, strategy: &BranchNameStrategy) -> Result<GitBranchName, MergeCommandResolveLocalBranchError> {
        use MergeCommandResolveLocalBranchError::*;
        let refs = handle!(git_refs(sh_dir), GitRefsFailed);
        let branch_name = handle!(strategy.to_branch_name("refs/heads", &refs), ToBranchNameFailed, strategy: strategy.to_owned());
        let exists = handle!(sh_dir.git_local_branch_exists(&branch_name), GitLocalBranchExistsFailed, branch_name);
        handle_bool!(!exists, LocalBranchDoesNotExist, branch_name);
        Ok(branch_name)
    }

    fn finish_pending_merge(sh_dir: &Shell, strategy: &BranchNameStrategy) -> Result<Option<GitBranchName>, MergeCommandFinishPendingMergeError> {
        use MergeCommandFinishPendingMergeError::*;
        let git_dir = handle!(cmd!(sh_dir, "git rev-parse --path-format=absolute --git-dir").read(), GitDirReadFailed);
        let git_dir = PathBuf::from(git_dir);
        let mut unexpected_state = [
            "rebase-merge",
            "rebase-apply",
            "CHERRY_PICK_HEAD",
            "REVERT_HEAD",
            "sequencer",
            "BISECT_START",
        ]
        .into_iter()
        .find(|state| sh_dir.path_exists(git_dir.join(state)));
        handle_opt_take!(unexpected_state, RepositoryStateInvalid, state, git_dir);
        let unmerged_paths = handle!(cmd!(sh_dir, "git diff --name-only --diff-filter=U").read(), UnmergedPathsReadFailed);
        handle_bool!(!unmerged_paths.is_empty(), UnresolvedConflicts, paths: unmerged_paths);
        if !sh_dir.path_exists(git_dir.join("MERGE_HEAD")) {
            return Ok(None);
        }
        let local_branch_name = handle!(Self::resolve_local_branch(sh_dir, strategy), ResolveLocalBranchFailed);
        let current_branch = handle!(cmd!(sh_dir, "git branch --show-current").read(), GitBranchFailed);
        handle_bool!(current_branch != local_branch_name, MergeBranchInvalid, current_branch, local_branch_name);
        handle!(Self::commit_merge(sh_dir), CommitMergeFailed);
        Ok(Some(local_branch_name))
    }

    fn pull(sh_dir: &Shell, local_branch_name: &str) -> Result<(), MergeCommandPullError> {
        use MergeCommandPullError::*;
        let upstream = handle!(
            cmd!(sh_dir, "git for-each-ref --format='%(upstream)' refs/heads/{local_branch_name}").read(),
            GitForEachRefFailed,
            local_branch_name: local_branch_name
        );
        handle_bool!(upstream.is_empty(), UpstreamNotFound, local_branch_name: local_branch_name);
        handle!(
            cmd!(sh_dir, "git pull --ff-only --no-rebase --no-squash").run_echo(),
            GitPullFailed,
            local_branch_name: local_branch_name,
            upstream
        );
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

        // Use `git merge --no-commit` + `git commit --no-edit` to trigger a pre-commit hook
        // Note that pre-merge-commit hook can't add files to the current git index, which means it can't update generated files (e.g. AGENTS.md or README.md)

        let flags = if allow_unrelated_histories {
            vec!["--allow-unrelated-histories", "--no-commit"]
        } else {
            vec!["--no-commit"]
        };

        handle!(cmd!(sh_dir, "git merge {remote}/{remote_branch_name} {flags...}").run_echo(), GitMergeFailed, remote, remote_branch_name);

        let merge_head_path = handle!(cmd!(sh_dir, "git rev-parse --path-format=absolute --git-path MERGE_HEAD").read(), GitMergeHeadPathFailed, remote, remote_branch_name);
        if !sh_dir.path_exists(merge_head_path) {
            return Ok(());
        }

        handle!(Self::commit_merge(sh_dir), CommitMergeFailed, remote, remote_branch_name);
        Ok(())
    }

    fn commit_merge(sh_dir: &Shell) -> Result<(), MergeCommandCommitMergeError> {
        use MergeCommandCommitMergeError::*;
        let pre_commit_path = sh_dir.current_dir().join(".repoconf/hooks/pre-commit.sh");
        handle!(Self::run_hook(sh_dir, pre_commit_path), RunHookFailed);
        handle!(cmd!(sh_dir, "git commit --no-edit").run_echo(), GitCommitFailed);
        Ok(())
    }

    fn run_hook(sh_dir: &Shell, path: PathBuf) -> Result<(), MergeCommandRunHookError> {
        use MergeCommandRunHookError::*;
        if !sh_dir.path_exists(&path) {
            return Ok(());
        }
        handle!(cmd!(sh_dir, "bash {path}").run_interactive(), RunInteractiveFailed, path);
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum MergeCommandRunError {
    #[error("failed to resolve the target directory")]
    UnwrapOrCurrentDirFailed { source: UnwrapOrCurrentDirError },
    #[error("failed to create a shell instance")]
    ShellNewFailed { source: xshell::Error },
    #[error("failed to finish a pending merge in '{dir}'", dir = dir.display())]
    FinishPendingMergeFailed { source: MergeCommandFinishPendingMergeError, dir: PathBuf },
    #[error("failed to read git remote names")]
    GitRemoteNamesFailed { source: GitRemoteNamesError },
    #[error("failed to check whether to skip repository '{dir}'", dir = dir.display())]
    ShouldSkipDirtyFailed { source: MergeCommandShouldSkipDirtyError, dir: PathBuf, is_clean: bool, allow_dirty: bool, skip_dirty: bool },
    #[error("failed to check repository status in '{dir}'", dir = dir.display())]
    IsCleanRepoFailed { source: IsCleanRepoError, dir: PathBuf },
    #[error("failed to read git refs after updating remotes")]
    GitRemoteRefsFailed { source: GitRefsError },
    #[error("failed to resolve the local branch in '{dir}'", dir = dir.display())]
    ResolveLocalBranchFailed { source: MergeCommandResolveLocalBranchError, dir: PathBuf },
    #[error("failed to check out local branch '{branch_name}'")]
    GitCheckoutFailed { source: xshell::Error, branch_name: String },
    #[error("failed to update branch '{branch_name}' from its upstream in '{dir}'", dir = dir.display())]
    PullFailed { source: MergeCommandPullError, dir: PathBuf, branch_name: String },
    #[error("failed to update repoconf remotes")]
    GitRemoteUpdateFailed { source: xshell::Error, remotes: Vec<String> },
    #[error("failed to merge remotes")]
    MergeRemotesFailed { source: MergeCommandMergeRemotesError },
    #[error("failed to run the post-merge hook")]
    RunHookFailed { source: MergeCommandRunHookError },
    #[error("failed to push merged changes")]
    GitPushFailed { source: xshell::Error },
}

#[derive(Error, Clone, Copy, Debug)]
pub enum MergeCommandShouldSkipDirtyError {
    #[error("repository has uncommitted changes; commit them or rerun with --allow-dirty or --skip-dirty")]
    RepositoryNotClean {},
}

#[derive(Error, Debug)]
pub enum MergeCommandResolveLocalBranchError {
    #[error("failed to read local git refs")]
    GitRefsFailed { source: GitRefsError },
    #[error("failed to resolve the local branch name")]
    ToBranchNameFailed { source: BranchNameStrategyToBranchNameError, strategy: BranchNameStrategy },
    #[error("failed to check whether local branch '{branch_name}' exists")]
    GitLocalBranchExistsFailed { source: GitLocalBranchExistsError, branch_name: String },
    #[error("local branch '{branch_name}' does not exist")]
    LocalBranchDoesNotExist { branch_name: String },
}

#[derive(Error, Debug)]
pub enum MergeCommandFinishPendingMergeError {
    #[error("failed to resolve the git directory")]
    GitDirReadFailed { source: xshell::Error },
    #[error("git state '{state}' in '{git_dir}' belongs to another unfinished operation; finish or abort that operation before merging templates", git_dir = git_dir.display())]
    RepositoryStateInvalid { state: String, git_dir: PathBuf },
    #[error("failed to resolve the destination branch before continuing the merge")]
    ResolveLocalBranchFailed { source: MergeCommandResolveLocalBranchError },
    #[error("failed to read the current branch before continuing the merge")]
    GitBranchFailed { source: xshell::Error },
    #[error("cannot continue a merge on branch '{current_branch}' while targeting '{local_branch_name}'; select the current branch with --local-branch or finish the pending merge separately")]
    MergeBranchInvalid { current_branch: String, local_branch_name: String },
    #[error("failed to read unresolved merge paths")]
    UnmergedPathsReadFailed { source: xshell::Error },
    #[error("merge conflicts remain:\n{paths}")]
    UnresolvedConflicts { paths: String },
    #[error("failed to commit the resolved merge")]
    CommitMergeFailed { source: MergeCommandCommitMergeError },
}

#[derive(Error, Debug)]
pub enum MergeCommandPullError {
    #[error("failed to read the upstream for local branch '{local_branch_name}'")]
    GitForEachRefFailed { source: xshell::Error, local_branch_name: String },
    #[error("local branch '{local_branch_name}' has no upstream; configure one with git branch --set-upstream-to or rerun with --no-pull")]
    UpstreamNotFound { local_branch_name: String },
    #[error("failed to pull upstream '{upstream}' into local branch '{local_branch_name}' with fast-forward-only updates")]
    GitPullFailed { source: xshell::Error, local_branch_name: String, upstream: String },
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
    #[error("failed to resolve the merge state path after merging from '{remote}/{remote_branch_name}'")]
    GitMergeHeadPathFailed { source: xshell::Error, remote: String, remote_branch_name: String },
    #[error("failed to commit the merge from '{remote}/{remote_branch_name}'")]
    CommitMergeFailed { source: MergeCommandCommitMergeError, remote: String, remote_branch_name: String },
}

#[derive(Error, Debug)]
pub enum MergeCommandCommitMergeError {
    #[error("failed to run the pre-commit hook")]
    RunHookFailed { source: MergeCommandRunHookError },
    #[error("failed to commit the merge")]
    GitCommitFailed { source: xshell::Error },
}

#[derive(Error, Debug)]
pub enum MergeCommandRunHookError {
    #[error("failed to run the repoconf hook '{path}'")]
    RunInteractiveFailed { source: xshell::Error, path: PathBuf },
}
