use crate::{Outcome, RepoName};
use clap::{value_parser, Parser};
use std::path::PathBuf;
use url::Url;
use xshell::{cmd, Shell};

#[derive(Parser, Clone, Debug)]
pub struct AddCommand {
    /// Template repo URL
    #[arg(value_parser = value_parser!(Url))]
    template: Url,

    /// Target repo directory
    #[arg(value_parser = value_parser!(PathBuf))]
    dir: PathBuf,
}

impl AddCommand {
    pub async fn run(self) -> Outcome {
        let Self {
            template,
            dir,
        } = self;

        let sh = Shell::new()?.with_current_dir(dir);

        let remote_template_name_suffix = template.repo_name();
        let remote_template_name = format!("repoconf-{remote_template_name_suffix}");
        let remote_template_url = template.as_str();

        cmd!(sh, "git remote add {remote_template_name} {remote_template_url}").run_echo()?;
        cmd!(sh, "git remote update {remote_template_name}").run_echo()?;

        Ok(())
    }
}
