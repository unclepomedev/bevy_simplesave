use crate::{
    SaveReadError, SaveWriteError,
    group::{ErasedSave, SaveEntry, SaveGroup, SaveMember, load_group_bag, save_group_bag},
};
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use std::{
    any::{TypeId, type_name},
    collections::HashMap,
    path::Path,
};

pub trait SaveGroupExt {
    /// Adds `R` as a member of group `G`. The resource itself is not created here:
    /// insert it yourself, or load it with [`load_group`].
    ///
    /// Panics if another member type of the same group already uses `R::KEY`.
    fn register_group_member<G: SaveGroup, R: SaveMember>(&mut self) -> &mut Self;
}

impl SaveGroupExt for App {
    fn register_group_member<G: SaveGroup, R: SaveMember>(&mut self) -> &mut Self {
        let mut group_members = self
            .world_mut()
            .get_resource_or_insert_with(GroupMembers::default);
        let members = group_members.by_group.entry(TypeId::of::<G>()).or_default();
        match members.iter().find(|e| e.key() == R::KEY) {
            Some(existing) if existing.member_type() == TypeId::of::<R>() => {}
            Some(_) => panic!(
                "bevy_simplesave: key `{}` of `{}` is already used by another member of group `{}`",
                R::KEY,
                type_name::<R>(),
                type_name::<G>()
            ),
            None => members.push(Box::new(SaveEntry::<R>::new())),
        }
        self
    }
}

/// Saves every resource registered to group `G` into a single file at `path`.
///
/// Fails with [`SaveWriteError::ResourceMissing`] if a member's resource does not exist;
/// nothing is written then. The file is replaced atomically.
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

/// Loads every member of group `G` from the file at `path`.
///
/// All-or-nothing: every member is decoded first and applied only if all are valid, so on failure the world is left untouched.
/// On success each member's resource is inserted, replacing an existing one. Keys in the file that no registered member uses are ignored.
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

/// The set of resource types registered to a single named group.
type GroupMemberList = Vec<Box<dyn ErasedSave>>;

#[derive(Resource, Default)]
struct GroupMembers {
    by_group: HashMap<TypeId, GroupMemberList>,
}

// ============================================================================================
// UNIT TESTS
// ============================================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::read_member;
    use serde::{Deserialize, Serialize};
    use std::{assert_matches, fs};

    struct SlotGroup;
    impl SaveGroup for SlotGroup {}

    impl SaveMember for Position {
        const KEY: &'static str = "position";
    }
    impl SaveMember for Health {
        const KEY: &'static str = "health";
    }
    impl SaveMember for Phase {
        const KEY: &'static str = "phase";
    }
    impl SaveMember for Misc {
        const KEY: &'static str = "misc";
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize, Resource)]
    struct Position {
        x: f32,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize, Resource)]
    struct Health {
        hp: u32,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Resource)]
    enum Phase {
        Intro,
        Chapter(u32),
        Scene { id: u32, done: bool },
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Resource)]
    struct Misc {
        maybe: Option<u32>,
        pair: (u8, u8),
        names: Vec<String>,
    }

    fn registered_app() -> App {
        let mut app = App::new();
        app.register_group_member::<SlotGroup, Position>();
        app.register_group_member::<SlotGroup, Health>();
        app
    }

    fn write_slot(path: &Path, x: f32, hp: u32) {
        let mut app = registered_app();
        app.insert_resource(Position { x });
        app.insert_resource(Health { hp });
        save_group::<SlotGroup>(app.world(), path).expect("save should succeed");
    }

    /// `Position` is valid, `Health` is broken.
    fn write_slot_with_broken_health(path: &Path) {
        let (position_key, health_key) = (Position::KEY, Health::KEY);
        fs::write(
            path,
            format!(r#"{{"{position_key}": (x: 9.0), "{health_key}": (hp: "bad")}}"#),
        )
        .unwrap();
    }

    #[test]
    fn load_group_applies_all_members() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("slot_1.ron");
        write_slot(&path, 3.0, 42);

        let mut app = registered_app();
        load_group::<SlotGroup>(app.world_mut(), &path).expect("load should succeed");

        assert_eq!(app.world().resource::<Position>().x, 3.0);
        assert_eq!(app.world().resource::<Health>().hp, 42);
    }

    #[test]
    fn load_group_fails_on_corrupt_member_and_leaves_world_untouched() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("slot_1.ron");
        write_slot_with_broken_health(&path);

        let mut app = registered_app();
        app.insert_resource(Position { x: 1.0 });

        let err = load_group::<SlotGroup>(app.world_mut(), &path).expect_err("must fail");
        assert_matches!(err, SaveReadError::GroupMemberDeserialize { .. });
        assert_eq!(app.world().resource::<Position>().x, 1.0);
        assert!(app.world().get_resource::<Health>().is_none());
    }

    #[test]
    fn load_group_fails_on_missing_member_and_leaves_world_untouched() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("slot_1.ron");

        // A file that only knows `Position`.
        let mut old_app = App::new();
        old_app.register_group_member::<SlotGroup, Position>();
        old_app.insert_resource(Position { x: 9.0 });
        save_group::<SlotGroup>(old_app.world(), &path).unwrap();

        let mut app = registered_app();
        app.insert_resource(Position { x: 1.0 });

        let err = load_group::<SlotGroup>(app.world_mut(), &path).expect_err("must fail");
        assert_matches!(err, SaveReadError::MissingMember { .. });
        assert_eq!(app.world().resource::<Position>().x, 1.0);
    }

    #[test]
    fn load_group_reports_missing_file_as_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("none.ron");

        let mut app = registered_app();
        let err = load_group::<SlotGroup>(app.world_mut(), &path).expect_err("must fail");
        assert!(err.is_not_found());
    }

    #[test]
    fn load_group_propagates_error_for_corrupt_bag_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("slot_1.ron");
        fs::write(&path, b"not valid ron {{{").unwrap();

        let mut app = registered_app();
        let err = load_group::<SlotGroup>(app.world_mut(), &path).expect_err("must fail");
        assert_matches!(err, SaveReadError::Deserialize(_));
    }

    #[test]
    fn registering_the_same_member_twice_is_harmless() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("slot_1.ron");

        let mut save_app = App::new();
        save_app.register_group_member::<SlotGroup, Position>();
        save_app.register_group_member::<SlotGroup, Position>();
        save_app.insert_resource(Position { x: 42.0 });
        save_group::<SlotGroup>(save_app.world(), &path).expect("save should succeed");

        let mut load_app = App::new();
        load_app.register_group_member::<SlotGroup, Position>();
        load_app.register_group_member::<SlotGroup, Position>();
        load_group::<SlotGroup>(load_app.world_mut(), &path).expect("load should succeed");

        assert_eq!(load_app.world().resource::<Position>().x, 42.0);
    }

    #[test]
    fn save_group_errors_when_a_member_resource_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("slot_1.ron");

        let app = registered_app(); // nothing inserted
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

        let app = App::new();
        let err = save_group::<NeverRegisteredGroup>(app.world(), &path)
            .expect_err("saving an unregistered group should fail");
        assert_matches!(err, SaveWriteError::UnknownGroup { .. });
    }

    #[test]
    fn read_member_reads_one_member_without_world_or_registration() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("slot_1.ron");
        write_slot(&path, 0.0, 42);

        let health = read_member::<Health>(&path).expect("should read");
        assert_eq!(health, Health { hp: 42 });
    }

    #[test]
    fn read_member_fails_when_the_key_is_absent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("slot_1.ron");

        let mut app = App::new();
        app.register_group_member::<SlotGroup, Position>();
        app.insert_resource(Position { x: 1.0 });
        save_group::<SlotGroup>(app.world(), &path).unwrap();

        let err = read_member::<Health>(&path).expect_err("Health is not in the file");
        assert_matches!(err, SaveReadError::MissingMember { .. });
    }

    #[test]
    fn read_member_fails_on_corrupt_member_but_other_members_stay_readable() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("slot_1.ron");
        write_slot_with_broken_health(&path);

        let err = read_member::<Health>(&path).expect_err("corrupt member");
        assert_matches!(err, SaveReadError::GroupMemberDeserialize { .. });
        assert_eq!(read_member::<Position>(&path).unwrap(), Position { x: 9.0 });
    }

    #[test]
    fn read_member_distinguishes_missing_file_from_corrupt_file() {
        let dir = tempfile::tempdir().unwrap();

        let missing = read_member::<Health>(dir.path().join("none.ron")).expect_err("missing");
        assert!(missing.is_not_found());

        let corrupt_path = dir.path().join("corrupt.ron");
        fs::write(&corrupt_path, b"not valid ron {{{").unwrap();
        let corrupt = read_member::<Health>(&corrupt_path).expect_err("corrupt");
        assert!(!corrupt.is_not_found());
        assert_matches!(corrupt, SaveReadError::Deserialize(_));
    }

    #[test]
    fn group_round_trip_preserves_enums_options_tuples_and_vecs() {
        let phases = [
            Phase::Intro,
            Phase::Chapter(3),
            Phase::Scene { id: 7, done: true },
        ];
        let misc_values = [
            Misc {
                maybe: None,
                pair: (1, 2),
                names: vec![],
            },
            Misc {
                maybe: Some(5),
                pair: (3, 4),
                names: vec!["a".into(), "b".into()],
            },
        ];

        for phase in &phases {
            for misc in &misc_values {
                let dir = tempfile::tempdir().unwrap();
                let path = dir.path().join("slot_1.ron");

                let mut save_app = App::new();
                save_app.register_group_member::<SlotGroup, Phase>();
                save_app.register_group_member::<SlotGroup, Misc>();
                save_app.insert_resource(phase.clone());
                save_app.insert_resource(misc.clone());
                save_group::<SlotGroup>(save_app.world(), &path).unwrap();

                let mut load_app = App::new();
                load_app.register_group_member::<SlotGroup, Phase>();
                load_app.register_group_member::<SlotGroup, Misc>();
                load_group::<SlotGroup>(load_app.world_mut(), &path)
                    .unwrap_or_else(|e| panic!("{phase:?} / {misc:?}: {e}"));

                assert_eq!(load_app.world().resource::<Phase>(), phase);
                assert_eq!(load_app.world().resource::<Misc>(), misc);
                assert_eq!(&read_member::<Phase>(&path).unwrap(), phase);
            }
        }
    }

    #[test]
    fn files_are_keyed_by_the_member_key_not_the_type_name() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("slot_1.ron");
        write_slot(&path, 1.0, 2);

        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains(r#""position""#) && text.contains(r#""health""#));
        assert!(!text.contains(type_name::<Position>()));
    }

    #[test]
    #[should_panic(expected = "already used")]
    fn registering_two_types_under_one_key_panics() {
        #[derive(Resource, Serialize, Deserialize)]
        struct Imposter {
            n: u8,
        }
        impl SaveMember for Imposter {
            const KEY: &'static str = "position";
        }

        let mut app = App::new();
        app.register_group_member::<SlotGroup, Position>();
        app.register_group_member::<SlotGroup, Imposter>();
    }
}
