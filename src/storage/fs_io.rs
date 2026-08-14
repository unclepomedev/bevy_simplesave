use crate::error::SaveError;
use std::fs;
use std::path::Path;

#[cfg_attr(not(test), expect(dead_code))]
pub(crate) fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), SaveError> {
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

#[cfg_attr(not(test), expect(dead_code))]
pub(crate) fn read_bytes(path: &Path) -> Result<Vec<u8>, SaveError> {
    fs::read(path).map_err(|source| SaveError::Io {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_bytes_creates_missing_parent_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let nested_path = dir.path().join("nested/dir/settings.ron");

        write_bytes(&nested_path, b"test").expect("should create parent dirs");
        assert!(nested_path.exists());
    }

    #[test]
    fn read_bytes_missing_file_returns_err() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("does_not_exist.ron");

        let result = read_bytes(&missing);
        assert!(result.is_err());
    }
}
