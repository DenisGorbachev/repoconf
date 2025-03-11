use crate::{Cmd, EnsureBranch, GitRemoteUrl, Outcome};
use clap::{value_parser, Parser};
use std::io;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

#[derive(Parser, Clone, Debug)]
pub struct EnsureCommand {
    #[arg(long, short)]
    parent_remote_name: String,

    #[arg(long, short = 'u')]
    parent_remote_url: GitRemoteUrl,

    /// Name of the remote GitHub repository
    #[arg(long, short = 'n')]
    repo_name: String,

    /// Owner of the remote GitHub repository
    #[arg(long, short, short = 'o')]
    repo_owner: String,

    /// Directory of the local repository
    #[arg(long, short, value_parser = value_parser!(PathBuf))]
    dir: Option<PathBuf>,
}

impl EnsureCommand {
    pub async fn run(self, _stdin: &mut impl Read, _stdout: &mut impl Write, _stderr: &mut impl Write) -> Outcome {
        let Self {
            dir,
            parent_remote_name,
            parent_remote_url,
            repo_name,
            repo_owner,
        } = self;

        /*
        for template_repo in template_repos {
            ensure_exists(template_repo.join(".repoconf/hooks/post-merge.sh"))?;
            ensure_exists(template_repo.join(".repoconf/hooks/post-clone-as-template.sh"))?;
            // ensure post-clone-as-template removes .repoconf
            // ensure post-merge removes .repoconf
        }

        For each dir in my projects:
          If it doesn't have a template:
            template = ask_template()
            // pipe stdin to repoconf/merge call
            repoconf/merge.sh {dir} {template}
        */

        let repo_name_short = format!("repoconf-{repo_name}");
        let repo_name_long = format!("{repo_owner}/{repo_name_short}");
        let repo_dir = dir.unwrap_or_else(|| PathBuf::from("/tmp").join(repo_name_short));

        if !gh_repo_exists(&repo_name_long)? {
            gh_repo_create(&repo_name_long)?;
        }

        gh_repo_clone(&repo_name_long, &repo_dir)?;

        repo_dir
            .cmd("git", &["commit", "--allow-empty", "-m", "Initial commit"])
            .run()?;

        repo_dir
            .cmd("git", &["push", "-u", "origin", "main"])
            .run()?;

        repo_dir.ensure_branch("main")?;
        repo_dir.ensure_branch("configs")?;
        repo_dir.ensure_branch("test")?;

        repo_dir
            .cmd("git", &["remote", "add", &parent_remote_name, &parent_remote_url])
            .run()?;

        Ok(())
    }
}

fn gh_repo_exists(repo_full_name: &str) -> Outcome<bool> {
    let output = Command::new("gh")
        .args(["repo", "view", repo_full_name])
        .status()?;

    Ok(output.success())
}

fn gh_repo_create(repo_full_name: &str) -> io::Result<ExitStatus> {
    Command::new("gh")
        .args(["repo", "create", repo_full_name])
        .status()
}

fn gh_repo_clone(repo_full_name: &str, repo_dir: &Path) -> io::Result<ExitStatus> {
    Command::new("gh")
        .args([
            "repo",
            "clone",
            repo_full_name,
            repo_dir.display().to_string().as_str(),
        ])
        .status()
}
