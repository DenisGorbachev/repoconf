use itertools::Itertools;
use xshell::{cmd, Shell};

pub fn git_refs(sh: &Shell) -> xshell::Result<Vec<String>> {
    let output = cmd!(sh, "git for-each-ref --format='%(refname)'").read()?;
    let vec = output.lines().map(ToOwned::to_owned).collect_vec();
    Ok(vec)
}
