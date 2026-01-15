use crate::{BranchNameStrategy, MergeCommand, MergeCommandRunError};
use clap::{value_parser, Parser};
use errgonomic::{handle, handle_iter, map_err, ErrVec};
use futures::stream::{self, TryStreamExt};
use itertools::Itertools;
use std::path::PathBuf;
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Parser, Clone, Debug)]
pub struct PropagateCommand {
    /// Name of the local branch to merge onto
    ///
    /// If you pass "-", the command will determine the branch automatically: use "main" if exists, use "master" if exists.
    ///
    /// If the local branch doesn't exist, the command will exit with an error
    ///
    /// The command will switch to this branch before merging
    #[arg(long, short, default_value = "-")]
    pub local_branch_name: BranchNameStrategy,

    /// Name of the remote branch to merge from
    ///
    /// If you pass "-", the command will determine the branch automatically: use "main" if exists, use "master" if exists.
    #[arg(long, short, default_value = "-")]
    pub remote_branch_name: BranchNameStrategy,

    /// Directory to search in, recursively
    #[arg(value_parser = value_parser!(PathBuf))]
    pub dir: PathBuf,
}

impl PropagateCommand {
    pub async fn run(self) -> Result<(), PropagateCommandRunError> {
        use PropagateCommandRunError::*;
        let Self {
            local_branch_name,
            remote_branch_name,
            dir,
        } = self;

        let repos = handle!(Self::collect_repos(&dir), CollectReposFailed, dir);
        handle!(Self::merge_repos(repos, local_branch_name, remote_branch_name).await, MergeReposFailed);

        Ok(())
    }

    fn collect_repos(dir: &PathBuf) -> Result<Vec<PathBuf>, PropagateCommandCollectReposError> {
        use PropagateCommandCollectReposError::*;
        let entries = handle_iter!(WalkDir::new(dir).into_iter(), WalkDirFailed, dir: dir);
        let repos = entries
            .into_iter()
            .map(|entry| entry.path().to_path_buf())
            .filter(|path| path.join(".git").exists())
            .collect_vec();
        Ok(repos)
    }

    async fn merge_repos(repos: Vec<PathBuf>, local_branch_name: BranchNameStrategy, remote_branch_name: BranchNameStrategy) -> Result<(), PropagateCommandMergeReposError> {
        use PropagateCommandMergeReposError::*;
        stream::iter(
            repos
                .into_iter()
                .map(Ok::<_, PropagateCommandMergeReposError>),
        )
        .try_for_each(|repo| async {
            println!("Entering {}", repo.display());
            let merge_command = MergeCommand {
                local_branch_strategy: local_branch_name.clone(),
                remote_branch_strategy: remote_branch_name.clone(),
                dir: Some(repo),
                ..MergeCommand::default()
            };
            map_err!(merge_command.run().await, MergeCommandRunFailed)
        })
        .await
    }
}

#[derive(Error, Debug)]
pub enum PropagateCommandRunError {
    #[error("failed to discover repositories under '{dir}'")]
    CollectReposFailed { source: PropagateCommandCollectReposError, dir: PathBuf },
    #[error("failed to merge discovered repositories")]
    MergeReposFailed { source: PropagateCommandMergeReposError },
}

#[derive(Error, Debug)]
pub enum PropagateCommandCollectReposError {
    #[error("failed to walk directory '{dir}'")]
    WalkDirFailed { source: ErrVec<walkdir::Error>, dir: PathBuf },
}

#[derive(Error, Debug)]
pub enum PropagateCommandMergeReposError {
    #[error("failed to merge repository")]
    MergeCommandRunFailed { source: MergeCommandRunError },
}
