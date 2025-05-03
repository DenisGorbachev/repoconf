use crate::{unwrap_or_current_dir, Outcome, RepoName};
use clap::{value_parser, Parser};
use std::path::PathBuf;
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
    pub async fn run(self) -> Outcome {
        let Self {
            template,
            dir,
        } = self;

        let dir = unwrap_or_current_dir(dir)?;
        let sh = Shell::new()?.with_current_dir(dir);

        let remote_template_name_suffix = template.repo_name();
        let remote_template_name = format!("repoconf-{remote_template_name_suffix}");
        let remote_template_url = template.as_str();

        cmd!(sh, "git remote add {remote_template_name} {remote_template_url}").run_echo()?;
        cmd!(sh, "git remote update {remote_template_name}").run_echo()?;

        Ok(())
    }
}
