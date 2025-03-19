use xshell::{cmd, Shell};

pub trait IsCleanRepo {
    type Output;

    fn is_clean_repo(&self) -> Self::Output;
}

impl IsCleanRepo for Shell {
    type Output = xshell::Result<bool>;

    fn is_clean_repo(&self) -> Self::Output {
        let output = cmd!(self, "git status --porcelain").read()?;
        Ok(output.is_empty())
    }
}
