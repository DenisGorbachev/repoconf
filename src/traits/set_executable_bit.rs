use std::fs::{metadata, set_permissions};
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

pub trait SetExecutableBit {
    type Output;

    fn set_executable_bit(self) -> Self::Output;
}

impl SetExecutableBit for &Path {
    type Output = io::Result<()>;

    fn set_executable_bit(self) -> Self::Output {
        // Retrieve file metadata
        let metadata = metadata(self)?;

        // Get current permissions
        let mut permissions = metadata.permissions();

        // Set execute permission for owner, group, and others
        let mode = permissions.mode() | 0o111; // Bitwise OR to add execute bits
        permissions.set_mode(mode);

        // Apply new permissions
        set_permissions(self, permissions)
    }
}

impl<'a> SetExecutableBit for &'a PathBuf {
    type Output = <&'a Path as SetExecutableBit>::Output;

    fn set_executable_bit(self) -> Self::Output {
        self.as_path().set_executable_bit()
    }
}
