use errgonomic::handle;
use itertools::Itertools;
use thiserror::Error;
use xshell::{cmd, Shell};

pub fn git_refs(sh: &Shell) -> Result<Vec<String>, GitRefsError> {
    use GitRefsError::*;
    let output = handle!(cmd!(sh, "git for-each-ref --format='%(refname)'").read(), ReadRefsFailed);
    let vec = output.lines().map(ToOwned::to_owned).collect_vec();
    Ok(vec)
}

#[derive(Error, Debug)]
pub enum GitRefsError {
    #[error("failed to read git refs")]
    ReadRefsFailed { source: xshell::Error },
}
