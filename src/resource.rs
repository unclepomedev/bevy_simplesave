use crate::storage;
use crate::storage::StorageError;
use crate::{SaveReadError, SaveWriteError};
use bevy_ecs::resource::Resource;
use bevy_ecs::world::World;
use serde::{Serialize, de::DeserializeOwned};
use std::any::type_name;
use std::io::ErrorKind;
use std::path::Path;

pub(crate) fn save_resource<R: Resource + Serialize>(
    world: &World,
    path: &Path,
) -> Result<(), SaveWriteError> {
    let resource = world
        .get_resource::<R>()
        .ok_or_else(|| SaveWriteError::ResourceMissing(type_name::<R>().to_string()))?;
    write_resource_ron(resource, path)
}

pub(crate) fn write_resource_ron<R: Serialize>(
    value: &R,
    path: &Path,
) -> Result<(), SaveWriteError> {
    let ron_str = storage::serialize_to_ron(value).map_err(write_err)?;
    storage::write_bytes(path, ron_str.as_bytes()).map_err(write_err)
}

/// Returns true if the resource was loaded from the file, false if the file did not exist.
pub(crate) fn load_resource<R: Resource + DeserializeOwned>(
    world: &mut World,
    path: &Path,
) -> Result<bool, SaveReadError> {
    let bytes = match storage::read_bytes(path) {
        Ok(bytes) => bytes,
        Err(StorageError::Io { source, .. }) if source.kind() == ErrorKind::NotFound => {
            return Ok(false);
        }
        Err(e) => return Err(read_err(e)),
    };
    let ron_str = String::from_utf8(bytes).map_err(SaveReadError::InvalidUtf8)?;
    let value: R = storage::deserialize_from_ron(&ron_str).map_err(read_err)?;
    world.insert_resource(value);
    Ok(true)
}

fn write_err(e: StorageError) -> SaveWriteError {
    match e {
        StorageError::Io { path, source } => SaveWriteError::Io { path, source },
        StorageError::Serialize(e) => SaveWriteError::Serialize(e),
        StorageError::Deserialize(_) => unreachable!("write path never deserializes"),
    }
}

fn read_err(e: StorageError) -> SaveReadError {
    match e {
        StorageError::Io { path, source } => SaveReadError::Io { path, source },
        StorageError::Deserialize(e) => SaveReadError::Deserialize(e),
        StorageError::Serialize(_) => unreachable!("read path never serializes"),
    }
}

// ============================================================================================
// UNIT TESTS
// ============================================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LoadFailed, SaveAppExt, SaveLocation, SavePlugin, SaveTiming};
    use bevy_app::App;
    use bevy_ecs::prelude::Messages;
    use serde::{Deserialize, Serialize};
    use std::assert_matches;
    use std::fs;

    #[derive(Debug, PartialEq, Serialize, Deserialize, Resource, Default)]
    struct DummySettings {
        volume: f32,
        difficulty: u8,
    }

    #[test]
    fn save_then_load_round_trip() {
        let dir = tempfile::tempdir().expect("tempdir should be created");
        let path = dir.path().join("settings.ron");

        let mut save_world = World::new();
        save_world.insert_resource(DummySettings {
            volume: 0.5,
            difficulty: 5,
        });
        save_resource::<DummySettings>(&save_world, &path).expect("save should succeed");
        assert!(path.exists());

        let mut load_world = World::new();
        load_resource::<DummySettings>(&mut load_world, &path).expect("load should succeed");

        let restored = load_world
            .get_resource::<DummySettings>()
            .expect("resource should have been inserted");
        assert_eq!(
            *restored,
            DummySettings {
                volume: 0.5,
                difficulty: 5,
            }
        );
    }

    #[test]
    fn save_resource_errors_when_resource_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.ron");

        let world = World::new(); // DummySettings not inserted
        let err = save_resource::<DummySettings>(&world, &path)
            .expect_err("save should fail when resource is missing");
        assert_matches!(err, SaveWriteError::ResourceMissing(_));
    }

    #[test]
    fn load_resource_is_noop_when_file_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("does_not_exist.ron");

        let mut world = World::new();
        load_resource::<DummySettings>(&mut world, &path)
            .expect("missing file should not be an error");

        assert!(world.get_resource::<DummySettings>().is_none());
    }

    #[test]
    fn load_resource_propagates_parse_errors() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("corrupt.ron");
        fs::write(&path, b"not valid ron {{{").unwrap();

        let mut world = World::new();
        let err = load_resource::<DummySettings>(&mut world, &path)
            .expect_err("corrupt file should be an error");
        assert_matches!(err, SaveReadError::Deserialize(_));
    }

    #[test]
    fn register_emits_load_failed_and_falls_back_to_default_on_corrupt_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.ron");
        fs::write(&path, b"not valid ron {{{").unwrap();

        let mut app = App::new();
        app.add_plugins(SavePlugin);
        app.register_saved_resource::<DummySettings>(
            SaveLocation::Custom(path),
            SaveTiming::Manual,
        );

        let resource = app.world().resource::<DummySettings>();
        assert_eq!(*resource, DummySettings::default());

        let messages = app.world().resource::<Messages<LoadFailed>>();
        assert_eq!(
            messages.len(),
            1,
            "corrupt save file should emit LoadFailed"
        );
    }
}
