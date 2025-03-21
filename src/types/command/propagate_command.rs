use crate::{BranchNameStrategy, MergeCommand, Outcome};
use clap::{value_parser, Parser};
use std::path::PathBuf;
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
    pub async fn run(self) -> Outcome {
        let Self {
            local_branch_name,
            remote_branch_name,
            dir,
        } = self;

        let repos = WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|entry| entry.path().join(".git").exists());

        // Iterate sequentially to preserve ordered output on stdout
        for repo in repos {
            println!("Entering {}", repo.path().display());
            let merge_command = MergeCommand {
                local_branch_strategy: local_branch_name.clone(),
                remote_branch_strategy: remote_branch_name.clone(),
                dir: Some(repo.path().to_path_buf()),
                ..MergeCommand::default()
            };
            merge_command.run().await?;
        }

        // let runs: JoinSet<Outcome> = repos
        //     .map(|entry| {
        //         let merge_command = MergeCommand {
        //             local_branch_name: local_branch_name.clone(),
        //             remote_branch_name: remote_branch_name.clone(),
        //             dir: entry.path().to_path_buf(),
        //         };
        //         merge_command.run()
        //     })
        //     .collect();
        //
        // let outcomes = runs.join_all().await;

        Ok(())
    }
}

pub fn has_repoconf_remotes() -> bool {
    todo!()
}
