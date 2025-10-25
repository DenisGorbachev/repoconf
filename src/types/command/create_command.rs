use crate::{DirectoryAlreadyExists, InitCommand, Outcome, RepoName, RepositoryAlreadyExists, Visibility};
use clap::{value_parser, Parser};
use std::path::PathBuf;
use url::Url;
use xshell::{cmd, Shell};

#[derive(Parser, Clone, Debug)]
pub struct CreateCommand {
    /// If the target repository exists, use it
    #[arg(long, short)]
    use_existing: bool,

    /// Don't run the post-init script
    #[arg(long, short)]
    skip_post_init: bool,

    /// Repo visibility
    #[arg(value_enum, long, short)]
    visibility: Visibility,

    /// Template repo URL
    #[arg(value_parser = value_parser!(Url))]
    template_url: Url,

    /// Owner of the new repository
    #[arg()]
    repo_owner: String,

    /// Name of the new repository
    #[arg()]
    repo_name: String,

    /// Name of the origin remote
    #[arg(long, short, default_value = "origin")]
    remote_name: String,

    /// Name of the main branch
    #[arg(long, short, default_value = "main")]
    branch_name: String,

    /// Directory to clone the new repository to
    #[arg(value_parser = value_parser!(PathBuf))]
    dir: PathBuf,
}

impl CreateCommand {
    pub async fn run(self) -> Outcome {
        let Self {
            use_existing,
            visibility,
            template_url,
            repo_owner,
            repo_name,
            remote_name,
            branch_name,
            skip_post_init,
            dir,
        } = self;

        let sh_cwd = Shell::new()?;

        let repo_name_full = format!("{repo_owner}/{repo_name}");
        // TODO: require passing template_name as arg
        let template_name = template_url.repo_name().to_string();
        let visibility_arg = visibility.as_arg();

        // TODO: This command fails with "401 Bad Credentials" when invoked through an alias (does it lose the environment?)
        let repo_view_cmd = cmd!(&sh_cwd, "gh repo view {repo_name_full}");
        eprintln!("$ {}", &repo_view_cmd);
        let mut repo_view_command = repo_view_cmd.to_command();
        let repo_view_output = repo_view_command.output()?;
        // dbg!(&repo_view_output);
        if repo_view_output.status.success() {
            if !use_existing {
                return Err(RepositoryAlreadyExists::new(repo_owner, repo_name).into());
            }
        } else {
            cmd!(sh_cwd, "gh repo create {visibility_arg} {repo_name_full}").run_echo()?;
        }

        if dir.try_exists()? {
            let sh_dir = sh_cwd.with_current_dir(&dir);
            let repo_name_full_current = cmd!(sh_dir, "gh repo view --json nameWithOwner --jq .nameWithOwner").read()?;
            if repo_name_full_current != repo_name_full {
                return Err(DirectoryAlreadyExists::new(dir).into());
            }
        } else {
            cmd!(sh_cwd, "gh repo clone {repo_name_full} {dir} -- --origin {remote_name}").run_echo()?;
        }

        let sh_dir = {
            let mut sh_dir = sh_cwd.with_current_dir(&dir);
            sh_dir.set_var("REPOCONF_VISIBILITY", visibility.to_string());
            sh_dir.set_var("REPOCONF_TEMPLATE", template_url.to_string());
            sh_dir.set_var("REPOCONF_REPO_OWNER", &repo_owner);
            sh_dir.set_var("REPOCONF_REPO_NAME", &repo_name);
            sh_dir
        };

        cmd!(sh_dir, "gh repo set-default {repo_name_full}").run_echo()?;

        let init_cmd = InitCommand {
            repo_name: repo_name.into(),
            remote_name,
            branch_name,
            skip_post_init,
            template_name,
            template_url,
            dir,
        };
        init_cmd.run().await?;

        Ok(())
    }
}
