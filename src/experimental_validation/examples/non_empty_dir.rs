//! This can be achieved with dedicated newtypes
//!
//! Verdict: this must be implemented in the following way:
//! * For each set of validators that can be applied to a single base type, define a single newtype that implements the following:
//!   * A dedicated `new` method that creates a value of newtype from base type, which runs all validators
//!   * TryFrom<WeakType> implementations for each weak type where a weak type is defined as a type with less than those validators
//!     * Note that the base type is a weak type, too (it has zero validators applied)

#![allow(dead_code)]

use errgonomic::handle;
use std::fs::read_dir;
use std::io;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use subtype::subtype_path_buf;
use thiserror::Error;

pub struct Valid<Value, Validator> {
    value: Value,
    validator: PhantomData<Validator>,
}

// TODO: IsDir checker
subtype_path_buf!(
    pub struct Dir(PathBuf);
);

// TODO: IsUtf8 checker
subtype_path_buf!(
    pub struct Utf8(PathBuf);
);

// TODO: IsUtf8 checker
subtype_path_buf!(
    pub struct NonEmpty(PathBuf);
);

pub fn is_dir(path: &Path) -> bool {
    path.is_dir()
}

pub fn is_utf8(path: &Path) -> bool {
    path.to_str().is_some()
}

// Some checks can be fallible, which adds another layer of complexity (we don't want to hide the errors from the caller)
pub fn is_non_empty(path: &Path) -> Result<bool, IsNonEmptyError> {
    use IsNonEmptyError::*;
    let path = path.to_path_buf();
    let mut entries = handle!(read_dir(&path), ReadDirFailed, path);
    Ok(entries.next().is_some())
}

#[derive(Error, Debug)]
pub enum IsNonEmptyError {
    #[error("failed to read directory '{path}'")]
    ReadDirFailed { source: io::Error, path: PathBuf },
}

// sigil struct
pub struct NonEmptyDir;

pub trait DirLike: AsRef<Path> {}
pub trait NonEmptyLike {}
pub trait NonEmptyDirLike: NonEmptyLike + DirLike {}

// NOTE: Impossible to implement
// pub fn to_dir_like(input: impl AsRef<Path>) -> Result<impl DirLike, ()> {
//     if input.as_ref().is_dir() {
//         Ok(input)
//     } else {
//         Err(())
//     }
// }

// Goal
pub fn get_files(_dir: impl NonEmptyDirLike) -> Vec<PathBuf> {
    todo!()
}
