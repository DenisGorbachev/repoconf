use errgonomic::handle_opt_take;
use std::path::{Path, PathBuf};
use thiserror::Error;
use xshell::Shell;

pub fn check_git_merge_state(sh_dir: &Shell, git_dir: &impl AsRef<Path>) -> Result<(), CheckGitMergeStateError> {
    use CheckGitMergeStateError::*;
    let git_dir = git_dir.as_ref();
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
    Ok(())
}

#[derive(Error, Debug)]
pub enum CheckGitMergeStateError {
    #[error("git state '{state}' in '{git_dir}' belongs to another unfinished operation; finish or abort that operation before merging templates", git_dir = git_dir.display())]
    RepositoryStateInvalid { state: String, git_dir: PathBuf },
}
