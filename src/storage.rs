mod fs_io;
mod ron_codec;

pub(crate) use fs_io::{read_bytes, write_bytes};
use ron::{Error as RonError, error::SpannedError};
pub(crate) use ron_codec::{deserialize_from_ron, serialize_to_ron};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::Error as IoError;
use std::path::PathBuf;

#[derive(Debug)]
pub(crate) enum StorageError {
    Io { path: PathBuf, source: IoError },
    Serialize(RonError),
    Deserialize(SpannedError),
}

impl Display for StorageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            StorageError::Io { path, source } => {
                write!(f, "io error at {}: {}", path.display(), source)
            }
            StorageError::Serialize(e) => write!(f, "failed to serialize to RON: {e}"),
            StorageError::Deserialize(e) => write!(f, "failed to deserialize from RON: {e}"),
        }
    }
}

// ============================================================================================
// UNIT TESTS
// ============================================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use fs_io::{read_bytes, write_bytes};
    use ron_codec::{deserialize_from_ron, serialize_to_ron};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct DummySettings {
        volume: f32,
        difficulty: u8,
    }

    #[test]
    fn file_round_trip() {
        let dir = tempfile::tempdir().expect("tempdir should be created");
        let path = dir.path().join("settings.ron");

        let original = DummySettings {
            volume: 0.5,
            difficulty: 5,
        };
        let ron_str = serialize_to_ron(&original).unwrap();

        write_bytes(&path, ron_str.as_bytes()).expect("write should succeed");
        assert!(path.exists());

        let bytes = read_bytes(&path).expect("read should succeed");
        let restored: DummySettings =
            deserialize_from_ron(&String::from_utf8(bytes).unwrap()).unwrap();

        assert_eq!(original, restored);
    }
}
