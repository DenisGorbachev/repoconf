use crate::{git_remote_exists, GitLocalBranchExists, Outcome, SetExecutableBit};
use clap::{value_parser, Parser};
use std::path::PathBuf;
use url::Url;
use xshell::{cmd, Shell};

#[derive(Parser, Clone, Debug)]
pub struct InitCommand {
    /// Name of the project (inferred from `dir` by default)
    #[arg(long, short = 'n')]
    pub repo_name: Option<String>,

    /// Name of the origin remote
    #[arg(long, short, default_value = "origin")]
    pub remote_name: String,

    /// Name of the main branch
    #[arg(long, short, default_value = "main")]
    pub branch_name: String,

    /// Don't run the post-init script
    #[arg(long, short)]
    pub skip_post_init: bool,

    /// Template repo name
    #[arg()]
    pub template_name: String,

    /// Template repo URL
    #[arg(value_parser = value_parser!(Url))]
    pub template_url: Url,

    /// Directory to clone the new repository to
    #[arg(value_parser = value_parser!(PathBuf))]
    pub dir: PathBuf,
}

impl InitCommand {
    pub async fn run(self) -> Outcome {
        let Self {
            repo_name,
            template_name,
            template_url,
            remote_name,
            branch_name,
            skip_post_init,
            dir,
        } = self;

        let sh_cwd = Shell::new()?;

        let repo_name = repo_name.unwrap_or_else(|| {
            dir.file_stem()
                .map(|os_str| os_str.to_str())
                .expect("dir should have a file stem")
                .expect("file stem should convert to string normally")
                .to_string()
        });
        let remote_template_name = format!("repoconf-{template_name}");
        let remote_template_url = template_url.as_str();

        let sh_dir = sh_cwd.with_current_dir(&dir);

        if !git_remote_exists(&sh_dir, remote_template_url)? {
            cmd!(sh_dir, "git remote add {remote_template_name} {remote_template_url}").run_echo()?;
        }
        cmd!(sh_dir, "git remote update {remote_template_name}").run_echo()?;

        if sh_dir.git_local_branch_exists(&branch_name)? {
            cmd!(sh_dir, "git checkout {branch_name}").run_echo()?;
        } else {
            cmd!(sh_dir, "git checkout -b {branch_name} {remote_template_name}/{branch_name}").run_echo()?;
            cmd!(sh_dir, "git branch --unset-upstream {branch_name}").run_echo()?;
        }

        cmd!(sh_dir, "git push --set-upstream {remote_name} {branch_name}").run_echo()?;

        if !skip_post_init {
            let post_init_script = sh_dir.current_dir().join(".repoconf/hooks/post-init.sh");
            post_init_script.set_executable_bit()?;
            if sh_dir.path_exists(&post_init_script) {
                cmd!(sh_cwd, "usage bash {post_init_script} --name {repo_name} {dir}").run_interactive()?;
            } else {
                eprintln!("Could not find post-init script at {post_init_script}", post_init_script = post_init_script.display());
            }
        }

        Ok(())
    }
}
