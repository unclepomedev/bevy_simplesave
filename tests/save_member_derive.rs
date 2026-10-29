use bevy_app::App;
use bevy_ecs::prelude::Resource;
use bevy_simplesave::{SaveGroup, SaveGroupExt, SaveMember, load_group, save_group};
use serde::{Deserialize, Serialize};

struct SlotGroup;
impl SaveGroup for SlotGroup {}

#[derive(Debug, PartialEq, Serialize, Deserialize, Resource, SaveMember)]
#[save(key = "derived")]
struct Derived {
    volume: f32,
}

#[test]
fn a_derived_member_round_trips_under_its_key() {
    assert_eq!(Derived::KEY, "derived");

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("slot.ron");

    let mut save_app = App::new();
    save_app.register_group_member::<SlotGroup, Derived>();
    save_app.insert_resource(Derived { volume: 0.5 });
    save_group::<SlotGroup>(save_app.world(), &path).expect("save should succeed");

    let mut load_app = App::new();
    load_app.register_group_member::<SlotGroup, Derived>();
    load_group::<SlotGroup>(load_app.world_mut(), &path).expect("load should succeed");

    assert_eq!(
        *load_app.world().resource::<Derived>(),
        Derived { volume: 0.5 }
    );
}
