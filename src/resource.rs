use crate::error::SaveError;
use crate::storage;
use bevy_ecs::resource::Resource;
use bevy_ecs::world::World;
use serde::{Serialize, de::DeserializeOwned};
use std::any::type_name;
use std::io::ErrorKind;
use std::path::Path;

#[cfg_attr(not(test), expect(dead_code))]
pub(crate) fn save_resource<R: Resource + Serialize>(
    world: &World,
    path: &Path,
) -> Result<(), SaveError> {
    let resource = world
        .get_resource::<R>()
        .ok_or_else(|| SaveError::ResourceMissing(type_name::<R>().to_string()))?;

    let ron_str = storage::serialize_to_ron(resource)?;
    storage::write_bytes(path, ron_str.as_bytes())
}

#[cfg_attr(not(test), expect(dead_code))]
pub(crate) fn load_resource<R: Resource + DeserializeOwned>(
    world: &mut World,
    path: &Path,
) -> Result<(), SaveError> {
    let bytes = match storage::read_bytes(path) {
        Ok(bytes) => bytes,
        Err(SaveError::Io { source, .. }) if source.kind() == ErrorKind::NotFound => {
            // No save file yet; leave the world untouched.
            return Ok(());
        }
        Err(e) => return Err(e),
    };

    let ron_str = String::from_utf8(bytes).map_err(SaveError::InvalidUtf8)?;
    let value: R = storage::deserialize_from_ron(&ron_str)?;
    world.insert_resource(value);
    Ok(())
}

// ============================================================================================
// UNIT TESTS
// ============================================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::assert_matches;

    #[derive(Debug, PartialEq, Serialize, Deserialize, Resource)]
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
        assert_matches!(err, SaveError::ResourceMissing(_));
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
        std::fs::write(&path, b"not valid ron {{{").unwrap();

        let mut world = World::new();
        let err = load_resource::<DummySettings>(&mut world, &path)
            .expect_err("corrupt file should be an error");
        assert_matches!(err, SaveError::Deserialize(_));
    }
}
