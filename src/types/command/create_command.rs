use crate::{Outcome, Visibility};
use clap::{value_parser, Parser};
use log::warn;
use std::io::{Read, Write};
use std::path::PathBuf;
use xshell::{cmd, Shell};

#[derive(Parser, Clone, Debug)]
pub struct CreateCommand {
    /// Repo visibility
    #[arg(value_enum, long, short)]
    visibility: Visibility,

    /// GitHub template repo URL
    #[arg()]
    template: String,

    /// Owner of the new repository
    #[arg()]
    repo_owner: String,

    /// Name of the new repository
    #[arg()]
    repo_name: String,

    /// Directory to clone the new repository to
    #[arg(value_parser = value_parser!(PathBuf))]
    dir: PathBuf,
}

// pub static SHELL: LazyLock<Shell> = LazyLock::new(|| Shell::new().expect("should create a new shell"));

impl CreateCommand {
    pub async fn run(self, _stdin: &mut impl Read, _stdout: &mut impl Write, _stderr: &mut impl Write) -> Outcome {
        let Self {
            visibility,
            template,
            repo_owner,
            repo_name,
            dir,
        } = self;

        let sh = Shell::new()?;

        let repo_name_full = format!("{repo_owner}/{repo_name}");
        let visibility_arg = visibility.as_arg();

        cmd!(sh, "gh repo create --template {template} {visibility_arg} {repo_name_full}").run_echo()?;
        cmd!(sh, "gh repo clone {repo_name_full} {dir}").run_echo()?;
        let sh_dir = sh.with_current_dir(&dir);
        cmd!(sh_dir, "git remote add template {template}").run_echo()?;
        let setup_script = sh_dir.current_dir().join(".repoconf/hooks/setup.sh");
        if sh_dir.path_exists(&setup_script) {
            cmd!(sh, ". {setup_script}").run_echo()?;
        } else {
            warn!("Setup script not found");
        }

        Ok(())
    }
}
