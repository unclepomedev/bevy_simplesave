use bevy_simplesave::{deserialize_from_ron, read_bytes, serialize_to_ron, write_bytes};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct DummySettings {
    volume: f32,
    difficulty: u8,
}

#[test]
fn ron_string_round_trip() {
    let original = DummySettings {
        volume: 0.8,
        difficulty: 2,
    };

    let ron_str = serialize_to_ron(&original).expect("serialize should succeed");
    let restored: DummySettings =
        deserialize_from_ron(&ron_str).expect("deserialize should succeed");

    assert_eq!(original, restored);
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
    let restored: DummySettings = deserialize_from_ron(&String::from_utf8(bytes).unwrap()).unwrap();

    assert_eq!(original, restored);
}

#[test]
fn write_bytes_creates_missing_parent_dirs() {
    let dir = tempfile::tempdir().unwrap();
    let nested_path = dir.path().join("nested/dir/settings.ron");

    write_bytes(&nested_path, b"test").expect("should create parent dirs");
    assert!(nested_path.exists());
}

#[test]
fn deserialize_invalid_ron_returns_err() {
    let result: Result<DummySettings, _> = deserialize_from_ron("not valid ron {{{");
    assert!(result.is_err());
}

#[test]
fn read_bytes_missing_file_returns_err() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("does_not_exist.ron");

    let result = read_bytes(&missing);
    assert!(result.is_err());
}
