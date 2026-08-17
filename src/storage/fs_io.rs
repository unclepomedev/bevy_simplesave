use super::StorageError;
use std::fs;
use std::io::{Error as IoError, ErrorKind, Write};
use std::path::Path;
use tempfile::NamedTempFile;

pub(crate) fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), StorageError> {
    let parent = path.parent().ok_or_else(|| StorageError::Io {
        path: path.to_path_buf(),
        source: IoError::new(ErrorKind::InvalidInput, "path has no parent directory"),
    })?;
    fs::create_dir_all(parent).map_err(|source| StorageError::Io {
        path: parent.to_path_buf(),
        source,
    })?;

    atomic_write(parent, path, bytes)
}

fn atomic_write(parent: &Path, path: &Path, bytes: &[u8]) -> Result<(), StorageError> {
    let mut tmp_file = NamedTempFile::new_in(parent).map_err(|source| StorageError::Io {
        path: parent.to_path_buf(),
        source,
    })?;
    tmp_file
        .write_all(bytes)
        .map_err(|source| StorageError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    tmp_file.persist(path).map_err(|e| StorageError::Io {
        path: path.to_path_buf(),
        source: e.error,
    })?;

    Ok(())
}

pub(crate) fn read_bytes(path: &Path) -> Result<Vec<u8>, StorageError> {
    fs::read(path).map_err(|source| StorageError::Io {
        path: path.to_path_buf(),
        source,
    })
}

// ============================================================================================
// UNIT TESTS
// ============================================================================================
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

    #[test]
    fn write_bytes_does_not_leave_temp_files_behind() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.ron");

        write_bytes(&path, b"content").expect("write should succeed");

        let entries: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();

        assert_eq!(
            entries,
            vec![std::ffi::OsString::from("settings.ron")],
            "only the final file should remain, no leftover temp files"
        );
    }

    #[test]
    fn write_bytes_overwrites_existing_file_content() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.ron");

        write_bytes(&path, b"old content").unwrap();
        write_bytes(&path, b"new content").unwrap();

        let content = fs::read(&path).unwrap();
        assert_eq!(content, b"new content");
    }
}
