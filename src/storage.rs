pub(crate) mod fs_io;
pub(crate) mod ron_codec;

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
