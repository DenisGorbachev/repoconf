use crate::{Outcome, SetExecutableBit, Visibility};
use clap::{value_parser, Parser};
use std::path::PathBuf;
use url::Url;
use xshell::{cmd, Shell};

#[derive(Parser, Clone, Debug)]
pub struct CreateCommand {
    /// Repo visibility
    #[arg(value_enum, long, short)]
    visibility: Visibility,

    /// GitHub template repo URL
    #[arg(value_parser = value_parser!(Url))]
    template: Url,

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
    pub async fn run(self) -> Outcome {
        let Self {
            visibility,
            template,
            repo_owner,
            repo_name,
            dir,
        } = self;

        let sh = Shell::new()?;

        let package_name = &repo_name;
        let repo_name_full = format!("{repo_owner}/{repo_name}");
        let remote_template_name = template
            .path_segments()
            .and_then(|mut split| split.next_back())
            .unwrap_or("template");
        let visibility_arg = visibility.as_arg();
        let template_str = template.as_str();

        cmd!(sh, "gh repo create --template {template_str} {visibility_arg} {repo_name_full}").run_echo()?;
        cmd!(sh, "gh repo clone {repo_name_full} {dir}").run_echo()?;
        let sh_dir = sh.with_current_dir(&dir);
        cmd!(sh_dir, "git remote add {remote_template_name} {template_str}").run_echo()?;
        let post_init_script = sh_dir.current_dir().join(".repoconf/hooks/post-init.sh");
        post_init_script.set_executable_bit()?;
        if sh_dir.path_exists(&post_init_script) {
            cmd!(sh, "usage bash {post_init_script} --name {package_name} {dir}").run_interactive()?;
        } else {
            eprintln!("Could not find post-init script at {post_init_script}", post_init_script = post_init_script.display());
        }

        Ok(())
    }
}
