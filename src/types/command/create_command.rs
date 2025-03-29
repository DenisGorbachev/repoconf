use crate::{git_remote_add_if_not_exists, DirectoryAlreadyExists, GitLocalBranchExists, Outcome, RepoName, RepositoryAlreadyExists, SetExecutableBit, Visibility};
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

    /// GitHub template repo URL
    #[arg(value_parser = value_parser!(Url))]
    template: Url,

    /// Owner of the new repository
    #[arg()]
    repo_owner: String,

    /// Name of the new repository
    #[arg()]
    repo_name: String,

    /// Name of the origin remote
    #[arg(long, short, default_value = "origin")]
    origin_remote_name: String,

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
            skip_post_init,
            visibility,
            template,
            repo_owner,
            repo_name,
            origin_remote_name,
            branch_name,
            dir,
        } = self;

        let sh_cwd = Shell::new()?;

        let package_name = &repo_name;
        let repo_name_full = format!("{repo_owner}/{repo_name}");
        let remote_template_name_suffix = template.repo_name();
        let remote_template_name = format!("repoconf-{remote_template_name_suffix}");
        let remote_template_url = template.as_str();
        let visibility_arg = visibility.as_arg();

        let repo_view_status = cmd!(&sh_cwd, "gh repo view {repo_name_full}")
            .to_command()
            .status()?;
        let repo_exists = repo_view_status.success();
        if repo_exists {
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
            cmd!(sh_cwd, "gh repo clone {repo_name_full} {dir} -- --origin {origin_remote_name}").run_echo()?;
        }

        let sh_dir = sh_cwd.with_current_dir(&dir);

        cmd!(sh_dir, "gh repo set-default {repo_name_full}").run_echo()?;

        git_remote_add_if_not_exists(&sh_dir, &remote_template_name, remote_template_url)?;
        cmd!(sh_dir, "git remote update {remote_template_name}").run_echo()?;

        if sh_dir.git_local_branch_exists(&branch_name)? {
            cmd!(sh_dir, "git checkout {branch_name}").run_echo()?;
        } else {
            cmd!(sh_dir, "git checkout -b {branch_name} {remote_template_name}/{branch_name}").run_echo()?;
            cmd!(sh_dir, "git branch --unset-upstream {branch_name}").run_echo()?;
        }

        cmd!(sh_dir, "git push --set-upstream {origin_remote_name} {branch_name}").run_echo()?;

        if !skip_post_init {
            let post_init_script = sh_dir.current_dir().join(".repoconf/hooks/post-init.sh");
            post_init_script.set_executable_bit()?;
            if sh_dir.path_exists(&post_init_script) {
                cmd!(sh_cwd, "usage bash {post_init_script} --name {package_name} {dir}").run_interactive()?;
            } else {
                eprintln!("Could not find post-init script at {post_init_script}", post_init_script = post_init_script.display());
            }
        }

        Ok(())
    }
}
