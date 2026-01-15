use crate::{git_remote_exists, GitLocalBranchExists, GitLocalBranchExistsError, GitRemoteExistsError, SetExecutableBit, SetExecutableBitError};
use clap::{value_parser, Parser};
use errgonomic::{handle, handle_opt};
use std::path::PathBuf;
use thiserror::Error;
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
    pub async fn run(self) -> Result<(), InitCommandRunError> {
        use InitCommandRunError::*;
        let Self {
            repo_name,
            template_name,
            template_url,
            remote_name,
            branch_name,
            skip_post_init,
            dir,
        } = self;

        let sh_cwd = handle!(Shell::new(), ShellNewFailed);

        let repo_name = match repo_name {
            Some(repo_name) => repo_name,
            None => {
                let file_stem = handle_opt!(dir.file_stem(), RepoNameNotFound, dir);
                let file_stem = handle_opt!(file_stem.to_str(), RepoNameNotUtf8, dir);
                file_stem.to_string()
            }
        };
        let remote_template_name = format!("repoconf-{template_name}");
        let remote_template_url = template_url.as_str();

        let sh_dir = sh_cwd.with_current_dir(&dir);

        let remote_exists = handle!(
            git_remote_exists(&sh_dir, remote_template_url),
            GitRemoteExistsFailed,
            remote_template_url: remote_template_url
        );
        if !remote_exists {
            handle!(
                cmd!(sh_dir, "git remote add {remote_template_name} {remote_template_url}").run_echo(),
                GitRemoteAddFailed,
                remote_template_name,
                remote_template_url: remote_template_url
            );
        }
        handle!(cmd!(sh_dir, "git remote update {remote_template_name}").run_echo(), GitRemoteUpdateFailed, remote_template_name);

        let local_branch_exists = handle!(sh_dir.git_local_branch_exists(&branch_name), GitLocalBranchExistsFailed, branch_name);
        if local_branch_exists {
            handle!(cmd!(sh_dir, "git checkout {branch_name}").run_echo(), GitCheckoutFailed, branch_name);
        } else {
            handle!(cmd!(sh_dir, "git checkout -b {branch_name} {remote_template_name}/{branch_name}").run_echo(), GitCheckoutNewBranchFailed, branch_name, remote_template_name);
            handle!(cmd!(sh_dir, "git branch --unset-upstream {branch_name}").run_echo(), GitBranchUnsetUpstreamFailed, branch_name);
        }

        handle!(cmd!(sh_dir, "git push --set-upstream {remote_name} {branch_name}").run_echo(), GitPushFailed, remote_name, branch_name);

        if !skip_post_init {
            let post_init_script = sh_dir.current_dir().join(".repoconf/hooks/post-init.sh");
            if sh_dir.path_exists(&post_init_script) {
                handle!(
                    post_init_script.set_executable_bit(),
                    SetExecutableBitFailed,
                    path: post_init_script
                );
                handle!(
                    cmd!(sh_dir, "usage bash {post_init_script} --name {repo_name} {dir}").run_interactive(),
                    PostInitRunFailed,
                    path: post_init_script,
                    repo_name,
                    dir
                );
            } else {
                eprintln!("[WARN] Could not find post-init script at {post_init_script}", post_init_script = post_init_script.display());
            }
        }

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum InitCommandRunError {
    #[error("failed to create a shell instance")]
    ShellNewFailed { source: xshell::Error },
    #[error("failed to infer repository name from '{dir}'")]
    RepoNameNotFound { dir: PathBuf },
    #[error("failed to parse repository name from '{dir}'")]
    RepoNameNotUtf8 { dir: PathBuf },
    #[error("failed to check whether template remote '{remote_template_url}' exists")]
    GitRemoteExistsFailed { source: GitRemoteExistsError, remote_template_url: String },
    #[error("failed to add git remote '{remote_template_name}' with url '{remote_template_url}'")]
    GitRemoteAddFailed { source: xshell::Error, remote_template_name: String, remote_template_url: String },
    #[error("failed to update git remote '{remote_template_name}'")]
    GitRemoteUpdateFailed { source: xshell::Error, remote_template_name: String },
    #[error("failed to check whether local branch '{branch_name}' exists")]
    GitLocalBranchExistsFailed { source: GitLocalBranchExistsError, branch_name: String },
    #[error("failed to check out local branch '{branch_name}'")]
    GitCheckoutFailed { source: xshell::Error, branch_name: String },
    #[error("failed to create local branch '{branch_name}' from '{remote_template_name}'")]
    GitCheckoutNewBranchFailed { source: xshell::Error, branch_name: String, remote_template_name: String },
    #[error("failed to unset upstream for branch '{branch_name}'")]
    GitBranchUnsetUpstreamFailed { source: xshell::Error, branch_name: String },
    #[error("failed to push branch '{branch_name}' to remote '{remote_name}'")]
    GitPushFailed { source: xshell::Error, remote_name: String, branch_name: String },
    #[error("failed to set executable bit for '{path}'")]
    SetExecutableBitFailed { source: SetExecutableBitError, path: PathBuf },
    #[error("failed to run post-init script '{path}' for '{repo_name}' in '{dir}'")]
    PostInitRunFailed { source: xshell::Error, path: PathBuf, repo_name: String, dir: PathBuf },
}
