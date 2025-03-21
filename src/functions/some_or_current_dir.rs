use std::env::current_dir;
use std::io;
use std::path::PathBuf;

pub fn some_or_current_dir(dir: Option<PathBuf>) -> io::Result<PathBuf> {
    match dir {
        None => current_dir(),
        Some(dir) => Ok(dir),
    }
}
