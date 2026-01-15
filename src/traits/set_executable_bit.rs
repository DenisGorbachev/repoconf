use errgonomic::handle;
use std::fs::{metadata, set_permissions};
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub trait SetExecutableBit {
    fn set_executable_bit(self) -> Result<(), SetExecutableBitError>;
}

impl SetExecutableBit for &Path {
    fn set_executable_bit(self) -> Result<(), SetExecutableBitError> {
        use SetExecutableBitError::*;
        let path = self.to_path_buf();
        let metadata = handle!(metadata(&path), MetadataFailed, path);

        // Get current permissions
        let mut permissions = metadata.permissions();

        // Set execute permission for owner, group, and others
        let mode = permissions.mode() | 0o111; // Bitwise OR to add execute bits
        permissions.set_mode(mode);

        // Apply new permissions
        handle!(set_permissions(&path, permissions), SetPermissionsFailed, path);
        Ok(())
    }
}

impl SetExecutableBit for &PathBuf {
    fn set_executable_bit(self) -> Result<(), SetExecutableBitError> {
        self.as_path().set_executable_bit()
    }
}

#[derive(Error, Debug)]
pub enum SetExecutableBitError {
    #[error("failed to read metadata for '{path}'")]
    MetadataFailed { source: io::Error, path: PathBuf },
    #[error("failed to set permissions for '{path}'")]
    SetPermissionsFailed { source: io::Error, path: PathBuf },
}
