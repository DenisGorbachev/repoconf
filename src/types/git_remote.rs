use crate::{GitRemoteName, GitRemoteUrl};
use derive_getters::Getters;
use derive_more::{Error, From, Into};
use derive_new::new;
use fmt_derive::Display;

#[derive(new, Getters, From, Into, Ord, PartialOrd, Eq, PartialEq, Default, Hash, Clone, Debug)]
pub struct GitRemote {
    pub name: GitRemoteName,
    pub url: GitRemoteUrl,
}

impl GitRemote {}

impl TryFrom<&str> for GitRemote {
    type Error = TryFromStrForGitRemoteError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use TryFromStrForGitRemoteError::*;
        let mut iter = value.split(char::is_whitespace).filter(|x| !x.is_empty());
        let name = iter.next().ok_or(NameNotFound)?;
        let url = iter.next().ok_or(UrlNotFound)?;
        Ok(Self {
            name: name.to_owned(),
            url: url.to_owned(),
        })
    }
}

#[derive(Error, Display, From, Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub enum TryFromStrForGitRemoteError {
    NameNotFound,
    UrlNotFound,
}
