use bevy_app::App;
use bevy_ecs::prelude::Resource;
use bevy_simplesave::{SaveAppExt, SaveLocation, SaveResource, SimpleSavePlugin};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Resource, Default, SaveResource)]
#[save(timing = manual)]
struct DerivedSettings {
    volume: f32,
}

#[test]
fn manual_timing_does_not_save_on_change_via_derive() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.ron");

    let mut app = App::new();
    app.add_plugins(SimpleSavePlugin);
    app.register_saveable::<DerivedSettings>(SaveLocation::Custom(path.clone()));

    app.world_mut().resource_mut::<DerivedSettings>().volume = 0.8;
    app.update();
    assert!(!path.exists(), "manual timing must not write on its own");

    bevy_simplesave::save_now::<DerivedSettings>(app.world()).expect("manual save should succeed");
    assert!(path.exists());
}
