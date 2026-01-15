use crate::{InitCommand, InitCommandRunError, RepoName, Visibility};
use clap::{value_parser, Parser};
use errgonomic::{handle, handle_bool};
use std::path::PathBuf;
use std::process::Output;
use thiserror::Error;
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
    pub async fn run(self) -> Result<(), CreateCommandRunError> {
        use CreateCommandRunError::*;
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

        let sh_cwd = handle!(Shell::new(), ShellNewFailed);

        let repo_name_full = format!("{repo_owner}/{repo_name}");
        let template_name = template_url.repo_name().to_string();
        let visibility_arg = visibility.as_arg();

        let repo_view_cmd = cmd!(&sh_cwd, "gh repo view --json name {repo_name_full}");
        eprintln!("$ {}", &repo_view_cmd);
        let mut repo_view_command = repo_view_cmd.to_command();
        let repo_view_output = handle!(repo_view_command.output(), RepoViewOutputFailed, repo_name_full);
        match repo_view_output.status.code() {
            Some(0) => {
                handle_bool!(!use_existing, RepositoryAlreadyExists, repo_name_full);
            }
            Some(1) => {
                let repository_not_found = repo_view_output
                    .stderr
                    .starts_with("GraphQL: Could not resolve to a Repository".as_bytes());
                if repository_not_found {
                    handle!(cmd!(sh_cwd, "gh repo create {visibility_arg} {repo_name_full}").run_echo(), RepoCreateFailed, repo_name_full, visibility_arg);
                } else {
                    return Err(RepoViewUnexpectedOutput {
                        repo_name_full,
                        output: repo_view_output,
                    });
                }
            }
            _ => {
                return Err(RepoViewUnexpectedOutput {
                    repo_name_full,
                    output: repo_view_output,
                });
            }
        }

        let dir_exists = handle!(dir.try_exists(), DirExistsCheckFailed, dir);
        if dir_exists {
            let sh_dir = sh_cwd.with_current_dir(&dir);
            let repo_name_full_current = handle!(cmd!(sh_dir, "gh repo view --json nameWithOwner --jq .nameWithOwner").read(), RepoNameWithOwnerReadFailed, dir);
            handle_bool!(repo_name_full_current != repo_name_full, DirectoryAlreadyExists, dir, repo_name_full, repo_name_full_current);
        } else {
            handle!(cmd!(sh_cwd, "gh repo clone {repo_name_full} {dir} -- --origin {remote_name}").run_echo(), RepoCloneFailed, repo_name_full, dir, remote_name);
        }

        let sh_dir = {
            let mut sh_dir = sh_cwd.with_current_dir(&dir);
            sh_dir.set_var("REPOCONF_VISIBILITY", visibility.to_string());
            sh_dir.set_var("REPOCONF_TEMPLATE", template_url.to_string());
            sh_dir.set_var("REPOCONF_REPO_OWNER", &repo_owner);
            sh_dir.set_var("REPOCONF_REPO_NAME", &repo_name);
            sh_dir
        };

        handle!(cmd!(sh_dir, "gh repo set-default {repo_name_full}").run_echo(), RepoSetDefaultFailed, repo_name_full);

        let init_cmd = InitCommand {
            repo_name: Some(repo_name),
            remote_name,
            branch_name,
            skip_post_init,
            template_name,
            template_url,
            dir,
        };
        handle!(init_cmd.run().await, InitCommandRunFailed, repo_name_full);

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum CreateCommandRunError {
    #[error("failed to create a shell instance")]
    ShellNewFailed { source: xshell::Error },
    #[error("failed to view repository '{repo_name_full}'")]
    RepoViewOutputFailed { source: std::io::Error, repo_name_full: String },
    #[error("repository '{repo_name_full}' already exists")]
    RepositoryAlreadyExists { repo_name_full: String },
    #[error("unexpected output while viewing repository '{repo_name_full}'")]
    RepoViewUnexpectedOutput { output: Output, repo_name_full: String },
    #[error("failed to create repository '{repo_name_full}' with visibility '{visibility_arg}'")]
    RepoCreateFailed { source: xshell::Error, repo_name_full: String, visibility_arg: String },
    #[error("failed to check whether directory '{dir}' exists")]
    DirExistsCheckFailed { source: std::io::Error, dir: PathBuf },
    #[error("failed to read repository name for directory '{dir}'")]
    RepoNameWithOwnerReadFailed { source: xshell::Error, dir: PathBuf },
    #[error("directory '{dir}' already exists and belongs to '{repo_name_full_current}'")]
    DirectoryAlreadyExists { dir: PathBuf, repo_name_full: String, repo_name_full_current: String },
    #[error("failed to clone repository '{repo_name_full}' into '{dir}' with remote '{remote_name}'")]
    RepoCloneFailed { source: xshell::Error, repo_name_full: String, dir: PathBuf, remote_name: String },
    #[error("failed to set default repository to '{repo_name_full}'")]
    RepoSetDefaultFailed { source: xshell::Error, repo_name_full: String },
    #[error("failed to initialize repository '{repo_name_full}'")]
    InitCommandRunFailed { source: InitCommandRunError, repo_name_full: String },
}
