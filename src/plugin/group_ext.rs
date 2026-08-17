use super::ensure_save_plugin_added;
use crate::group::{ErasedSave, SaveEntry, load_group_bag, save_group_bag};
use crate::{SaveReadError, SaveWriteError};
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use serde::{Serialize, de::DeserializeOwned};
use std::any::type_name;
use std::collections::HashMap;
use std::path::Path;

#[derive(Resource, Default)]
struct GroupMembers(HashMap<&'static str, Vec<Box<dyn ErasedSave>>>);

pub trait SaveGroupExt {
    fn register_group_member<R>(&mut self, group: &'static str) -> &mut Self
    where
        R: Resource + Serialize + DeserializeOwned + Default;
}

impl SaveGroupExt for App {
    /// Panics if [`crate::SavePlugin`] has not been added to the app.
    fn register_group_member<R>(&mut self, group: &'static str) -> &mut Self
    where
        R: Resource + Serialize + DeserializeOwned + Default,
    {
        ensure_save_plugin_added(self.world());

        let world = self.world_mut();
        if world.get_resource::<R>().is_none() {
            world.insert_resource(R::default());
        }
        let mut members = world.get_resource_or_insert_with(GroupMembers::default);
        let group_entries = members.0.entry(group).or_default();
        let key = type_name::<R>();
        if !group_entries.iter().any(|e| e.type_key() == key) {
            group_entries.push(Box::new(SaveEntry::<R>::new()));
        }
        self
    }
}

/// Saves every resource registered to `group` into a single file at `path`.
pub fn save_group(world: &World, group: &'static str, path: &Path) -> Result<(), SaveWriteError> {
    let entries = world
        .get_resource::<GroupMembers>()
        .and_then(|m| m.0.get(group))
        .ok_or_else(|| SaveWriteError::UnknownGroup(group.to_string()))?;
    save_group_bag(world, entries, path)
}

/// Loads every resource registered to `group` from a single file at `path`.
pub fn load_group(
    world: &mut World,
    group: &'static str,
    path: &Path,
) -> Result<(), SaveReadError> {
    if !world.contains_resource::<GroupMembers>() {
        return Err(SaveReadError::UnknownGroup(group.to_string()));
    }
    world.resource_scope(|world, members: Mut<GroupMembers>| {
        let entries = members
            .0
            .get(group)
            .ok_or_else(|| SaveReadError::UnknownGroup(group.to_string()))?;
        load_group_bag(world, entries, path)
    })
}

// ============================================================================================
// UNIT TESTS
// ============================================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::SavePlugin;
    use serde::{Deserialize, Serialize};

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
        save_app.add_plugins(SavePlugin);
        save_app.register_group_member::<Position>("slot");
        save_app.register_group_member::<Health>("slot");
        save_app.world_mut().resource_mut::<Position>().x = 3.0;
        save_app.world_mut().resource_mut::<Health>().hp = 42;

        save_group(save_app.world(), "slot", &path).expect("save should succeed");

        let mut load_app = App::new();
        load_app.add_plugins(SavePlugin);
        load_app.register_group_member::<Position>("slot");
        load_app.register_group_member::<Health>("slot");
        load_group(load_app.world_mut(), "slot", &path).expect("load should succeed");

        assert_eq!(load_app.world().resource::<Position>().x, 3.0);
        assert_eq!(load_app.world().resource::<Health>().hp, 42);
    }

    #[test]
    fn load_group_defaults_newly_added_member_when_key_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("slot_1.ron");

        // Old save: only Position was known at the time.
        let mut old_app = App::new();
        old_app.add_plugins(SavePlugin);
        old_app.register_group_member::<Position>("slot");
        old_app.world_mut().resource_mut::<Position>().x = 7.0;
        save_group(old_app.world(), "slot", &path).unwrap();

        // Newer build adds Health to the group.
        let mut new_app = App::new();
        new_app.add_plugins(SavePlugin);
        new_app.register_group_member::<Position>("slot");
        new_app.register_group_member::<Health>("slot");
        load_group(new_app.world_mut(), "slot", &path).expect("load should succeed");

        assert_eq!(new_app.world().resource::<Position>().x, 7.0);
        assert_eq!(*new_app.world().resource::<Health>(), Health::default());
    }

    #[test]
    fn register_same_resource_twice_preserves_loaded_value() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("slot_duplicate.ron");

        let mut save_app = App::new();
        save_app.add_plugins(SavePlugin);
        save_app.register_group_member::<Position>("slot");
        save_app.register_group_member::<Position>("slot");
        save_app.world_mut().resource_mut::<Position>().x = 42.0;

        save_group(save_app.world(), "slot", &path).expect("save should succeed");

        let mut load_app = App::new();
        load_app.add_plugins(SavePlugin);
        load_app.register_group_member::<Position>("slot");
        load_app.register_group_member::<Position>("slot");
        load_group(load_app.world_mut(), "slot", &path).expect("load should succeed");

        assert_eq!(load_app.world().resource::<Position>().x, 42.0);
    }
}
