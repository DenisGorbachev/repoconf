use duct::{cmd, Expression, IntoExecutablePath};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub trait Cmd {
    fn cmd<T, U>(&self, program: T, args: U) -> Expression
    where
        T: IntoExecutablePath,
        U: IntoIterator,
        U::Item: Into<OsString>;
}

impl Cmd for PathBuf {
    fn cmd<T, U>(&self, program: T, args: U) -> Expression
    where
        T: IntoExecutablePath,
        U: IntoIterator,
        U::Item: Into<OsString>,
    {
        cmd(program, args).dir(self)
    }
}
