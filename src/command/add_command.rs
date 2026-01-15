use crate::{unwrap_or_current_dir, RepoName, UnwrapOrCurrentDirError};
use clap::{value_parser, Parser};
use errgonomic::handle;
use std::path::PathBuf;
use thiserror::Error;
use url::Url;
use xshell::{cmd, Shell};

#[derive(Parser, Clone, Debug)]
pub struct AddCommand {
    /// Target repo directory (defaults to current dir)
    #[arg(long, short, value_parser = value_parser!(PathBuf))]
    dir: Option<PathBuf>,

    /// Template repo URL
    #[arg(value_parser = value_parser!(Url))]
    template: Url,
}

impl AddCommand {
    pub async fn run(self) -> Result<(), AddCommandRunError> {
        use AddCommandRunError::*;
        let Self {
            template,
            dir,
        } = self;

        let dir = handle!(unwrap_or_current_dir(dir), UnwrapOrCurrentDirFailed);
        let sh = handle!(Shell::new(), ShellNewFailed);
        let sh = sh.with_current_dir(dir);

        let remote_template_name_suffix = template.repo_name();
        let remote_template_name = format!("repoconf-{remote_template_name_suffix}");
        let remote_template_url = template.as_str();

        handle!(
            cmd!(sh, "git remote add {remote_template_name} {remote_template_url}").run_echo(),
            GitRemoteAddFailed,
            remote_template_name,
            remote_template_url: remote_template_url
        );
        handle!(cmd!(sh, "git remote update {remote_template_name}").run_echo(), GitRemoteUpdateFailed, remote_template_name);

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum AddCommandRunError {
    #[error("failed to resolve the target directory")]
    UnwrapOrCurrentDirFailed { source: UnwrapOrCurrentDirError },
    #[error("failed to create a shell instance")]
    ShellNewFailed { source: xshell::Error },
    #[error("failed to add git remote '{remote_template_name}' with url '{remote_template_url}'")]
    GitRemoteAddFailed { source: xshell::Error, remote_template_name: String, remote_template_url: String },
    #[error("failed to update git remote '{remote_template_name}'")]
    GitRemoteUpdateFailed { source: xshell::Error, remote_template_name: String },
}
