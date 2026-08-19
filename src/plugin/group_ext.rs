use super::ensure_save_plugin_added;
use crate::group::{ErasedSave, SaveEntry, SaveGroup, load_group_bag, save_group_bag};
use crate::{SaveReadError, SaveWriteError};
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use serde::{Serialize, de::DeserializeOwned};
use std::any::{TypeId, type_name};
use std::collections::HashMap;
use std::path::Path;

/// The set of resource types registered to a single named group.
type GroupMemberList = Vec<Box<dyn ErasedSave>>;

#[derive(Resource, Default)]
struct GroupMembers {
    by_group: HashMap<TypeId, GroupMemberList>,
}

pub trait SaveGroupExt {
    /// Adds `R` as a member of group `G`. Ensures a default value of `R`
    /// exists in the world immediately; call [`load_group`] to populate it from a file.
    ///
    /// Panics if [`crate::SimpleSavePlugin`] has not been added to the app.
    fn register_group_member<G: SaveGroup, R>(&mut self) -> &mut Self
    where
        R: Resource + Serialize + DeserializeOwned + Default;
}

impl SaveGroupExt for App {
    fn register_group_member<G: SaveGroup, R>(&mut self) -> &mut Self
    where
        R: Resource + Serialize + DeserializeOwned + Default,
    {
        ensure_save_plugin_added(self.world());

        let world = self.world_mut();
        if world.get_resource::<R>().is_none() {
            world.insert_resource(R::default());
        }
        let key = type_name::<R>();
        let mut group_members = world.get_resource_or_insert_with(GroupMembers::default);
        let members = group_members.by_group.entry(TypeId::of::<G>()).or_default();
        if !members.iter().any(|e| e.type_key() == key) {
            members.push(Box::new(SaveEntry::<R>::new()));
        }
        self
    }
}

/// Saves every resource registered to group `G` into a single file at `path`.
pub fn save_group<G: SaveGroup>(
    world: &World,
    path: impl AsRef<Path>,
) -> Result<(), SaveWriteError> {
    let entries = world
        .get_resource::<GroupMembers>()
        .and_then(|m| m.by_group.get(&TypeId::of::<G>()))
        .ok_or_else(|| SaveWriteError::UnknownGroup {
            group: type_name::<G>().to_string(),
        })?;
    save_group_bag(world, entries, path.as_ref())
}

/// Loads every resource registered to group `G` from a single file at `path`.
pub fn load_group<G: SaveGroup>(
    world: &mut World,
    path: impl AsRef<Path>,
) -> Result<(), SaveReadError> {
    if !world.contains_resource::<GroupMembers>() {
        return Err(SaveReadError::UnknownGroup {
            group: type_name::<G>().to_string(),
        });
    }
    world.resource_scope(|world, members: Mut<GroupMembers>| {
        let entries = members.by_group.get(&TypeId::of::<G>()).ok_or_else(|| {
            SaveReadError::UnknownGroup {
                group: type_name::<G>().to_string(),
            }
        })?;
        load_group_bag(world, entries, path.as_ref())
    })
}

// ============================================================================================
// UNIT TESTS
// ============================================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LoadFailed, SimpleSavePlugin};
    use serde::{Deserialize, Serialize};
    use std::assert_matches;
    use std::fs;

    struct SlotGroup;
    impl SaveGroup for SlotGroup {}

    #[derive(Debug, PartialEq, Serialize, Deserialize, Resource, Default)]
    struct Position {
        x: f32,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize, Resource, Default)]
    struct Health {
        hp: u32,
    }

    #[test]
    fn group_round_trip_with_two_resource_types() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("slot_1.ron");

        let mut save_app = App::new();
        save_app.add_plugins(SimpleSavePlugin);
        save_app.register_group_member::<SlotGroup, Position>();
        save_app.register_group_member::<SlotGroup, Health>();
        save_app.world_mut().resource_mut::<Position>().x = 3.0;
        save_app.world_mut().resource_mut::<Health>().hp = 42;

        save_group::<SlotGroup>(save_app.world(), &path).expect("save should succeed");

        let mut load_app = App::new();
        load_app.add_plugins(SimpleSavePlugin);
        load_app.register_group_member::<SlotGroup, Position>();
        load_app.register_group_member::<SlotGroup, Health>();
        load_group::<SlotGroup>(load_app.world_mut(), &path).expect("load should succeed");

        assert_eq!(load_app.world().resource::<Position>().x, 3.0);
        assert_eq!(load_app.world().resource::<Health>().hp, 42);
    }

    #[test]
    fn load_group_defaults_newly_added_member_when_key_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("slot_1.ron");

        // Old save: only Position was known at the time.
        let mut old_app = App::new();
        old_app.add_plugins(SimpleSavePlugin);
        old_app.register_group_member::<SlotGroup, Position>();
        old_app.world_mut().resource_mut::<Position>().x = 7.0;
        save_group::<SlotGroup>(old_app.world(), &path).unwrap();

        // Newer build adds Health to the group.
        let mut new_app = App::new();
        new_app.add_plugins(SimpleSavePlugin);
        new_app.register_group_member::<SlotGroup, Position>();
        new_app.register_group_member::<SlotGroup, Health>();
        load_group::<SlotGroup>(new_app.world_mut(), &path).expect("load should succeed");

        assert_eq!(new_app.world().resource::<Position>().x, 7.0);
        assert_eq!(*new_app.world().resource::<Health>(), Health::default());
    }

    #[test]
    fn register_same_resource_twice_preserves_loaded_value() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("slot_duplicate.ron");

        let mut save_app = App::new();
        save_app.add_plugins(SimpleSavePlugin);
        save_app.register_group_member::<SlotGroup, Position>();
        save_app.register_group_member::<SlotGroup, Position>();
        save_app.world_mut().resource_mut::<Position>().x = 42.0;

        save_group::<SlotGroup>(save_app.world(), &path).expect("save should succeed");

        let mut load_app = App::new();
        load_app.add_plugins(SimpleSavePlugin);
        load_app.register_group_member::<SlotGroup, Position>();
        load_app.register_group_member::<SlotGroup, Position>();
        load_group::<SlotGroup>(load_app.world_mut(), &path).expect("load should succeed");

        assert_eq!(load_app.world().resource::<Position>().x, 42.0);
    }

    #[test]
    fn load_group_propagates_error_for_corrupt_bag_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("slot_1.ron");
        fs::write(&path, b"not valid ron {{{").unwrap();

        let mut app = App::new();
        app.add_plugins(SimpleSavePlugin);
        app.register_group_member::<SlotGroup, Position>();

        let err = load_group::<SlotGroup>(app.world_mut(), &path)
            .expect_err("corrupt bag file should be an error");
        assert_matches!(err, SaveReadError::Deserialize(_));
    }

    #[test]
    fn load_group_defaults_one_corrupt_member_but_loads_others() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("slot_1.ron");

        let position_key = type_name::<Position>();
        let health_key = type_name::<Health>();
        let bag_ron =
            format!(r#"{{"{position_key}": (x: "not_a_number"), "{health_key}": (hp: 42)}}"#);
        fs::write(&path, bag_ron).unwrap();

        let mut app = App::new();
        app.add_plugins(SimpleSavePlugin);
        app.register_group_member::<SlotGroup, Position>();
        app.register_group_member::<SlotGroup, Health>();

        load_group::<SlotGroup>(app.world_mut(), &path).expect("load should succeed overall");

        assert_eq!(*app.world().resource::<Position>(), Position::default());
        assert_eq!(app.world().resource::<Health>().hp, 42);

        let messages = app.world().resource::<Messages<LoadFailed>>();
        assert_eq!(
            messages.len(),
            1,
            "corrupt member should emit exactly one LoadFailed"
        );
        let msg = messages.iter_current_update_messages().next().unwrap();
        assert_eq!(msg.resource_type, type_name::<Position>());
        assert_matches!(msg.error, SaveReadError::GroupMemberDeserialize(_));
    }

    #[test]
    fn save_group_errors_when_a_member_resource_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("slot_1.ron");

        let mut app = App::new();
        app.add_plugins(SimpleSavePlugin);
        app.register_group_member::<SlotGroup, Position>();
        app.world_mut().remove_resource::<Position>();

        let err = save_group::<SlotGroup>(app.world(), &path)
            .expect_err("save should fail when a member resource is missing");
        assert_matches!(err, SaveWriteError::ResourceMissing { .. });
        assert!(!path.exists(), "no partial file should be written");
    }

    #[test]
    fn save_write_error_serialize_display_formats_resource_type_and_source() {
        let err = SaveWriteError::Serialize {
            resource_type: "MyResource".to_string(),
            source: ron::Error::Message("custom ron error".to_string()),
        };
        assert_eq!(
            err.to_string(),
            "failed to serialize `MyResource` to RON: custom ron error"
        );
    }

    #[test]
    fn save_group_errors_when_group_was_never_registered() {
        struct NeverRegisteredGroup;
        impl SaveGroup for NeverRegisteredGroup {}

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("slot_1.ron");

        let mut app = App::new();
        app.add_plugins(SimpleSavePlugin);

        let err = save_group::<NeverRegisteredGroup>(app.world(), &path)
            .expect_err("saving an unregistered group should fail");
        assert_matches!(err, SaveWriteError::UnknownGroup { .. });
    }
}
