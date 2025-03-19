use crate::GitRemoteName;
use std::io;
use std::path::{Path, PathBuf};

pub trait GitRemoteNames {
    type Output;

    fn git_remote_names(&self) -> Self::Output;
}

impl GitRemoteNames for &Path {
    type Output = io::Result<Vec<GitRemoteName>>;

    fn git_remote_names(&self) -> Self::Output {
        todo!()
    }
}

impl<'a> GitRemoteNames for &'a PathBuf {
    type Output = <&'a Path as GitRemoteNames>::Output;

    fn git_remote_names(&self) -> Self::Output {
        self.as_path().git_remote_names()
    }
}
