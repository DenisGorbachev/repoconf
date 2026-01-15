use crate::{GitRemoteName, GitRemoteUrl};
use derive_getters::Getters;
use derive_new::new;
use errgonomic::handle_opt;
use thiserror::Error;

#[derive(new, Getters, Ord, PartialOrd, Eq, PartialEq, Default, Hash, Clone, Debug)]
pub struct GitRemote {
    pub name: GitRemoteName,
    pub url: GitRemoteUrl,
}

impl GitRemote {}

impl TryFrom<&str> for GitRemote {
    type Error = ConvertStrToGitRemoteError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use ConvertStrToGitRemoteError::*;
        let input = value.to_string();
        let mut iter = value.split(char::is_whitespace).filter(|x| !x.is_empty());
        let name = handle_opt!(iter.next(), NameNotFound, input);
        let url = handle_opt!(iter.next(), UrlNotFound, input);
        Ok(Self {
            name: name.to_owned(),
            url: url.to_owned(),
        })
    }
}

#[derive(Error, Debug)]
pub enum ConvertStrToGitRemoteError {
    #[error("git remote name not found in '{input}'")]
    NameNotFound { input: String },
    #[error("git remote url not found in '{input}'")]
    UrlNotFound { input: String },
}
