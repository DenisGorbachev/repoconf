use derive_more::{Error, From, Into};
use derive_new::new;
use fmt_derive::Display;
use std::path::PathBuf;

#[derive(new, Error, Display, From, Into, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Debug)]
pub struct DirectoryAlreadyExists {
    dir: PathBuf,
}

impl DirectoryAlreadyExists {}
