use errgonomic::handle;
use std::env::current_dir;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

pub fn unwrap_or_current_dir(dir: Option<PathBuf>) -> Result<PathBuf, UnwrapOrCurrentDirError> {
    use UnwrapOrCurrentDirError::*;
    match dir {
        None => Ok(handle!(current_dir(), CurrentDirFailed)),
        Some(dir) => Ok(dir),
    }
}

#[derive(Error, Debug)]
pub enum UnwrapOrCurrentDirError {
    #[error("failed to get the current directory")]
    CurrentDirFailed { source: io::Error },
}
