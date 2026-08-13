use crate::error::SaveError;
use std::fs;
use std::path::Path;

pub fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), SaveError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| SaveError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    fs::write(path, bytes).map_err(|source| SaveError::Io {
        path: path.to_path_buf(),
        source,
    })
}

pub fn read_bytes(path: &Path) -> Result<Vec<u8>, SaveError> {
    fs::read(path).map_err(|source| SaveError::Io {
        path: path.to_path_buf(),
        source,
    })
}
