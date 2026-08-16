mod app_ext;
mod save_now;
mod save_path;
mod systems;
mod timing;

pub use self::app_ext::SaveAppExt;
pub use self::save_now::save_now;
pub(crate) use self::save_path::SavePath;
pub use self::timing::SaveTiming;
use crate::{LoadFailed, SaveFailed};
use bevy_app::prelude::*;

#[derive(Default)]
pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SaveFailed>();
        app.add_message::<LoadFailed>();
    }
}

// ============================================================================================
// UNIT TESTS
// ============================================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::SaveLocation;
    use bevy_ecs::prelude::*;
    use serde::{Deserialize, Serialize};
    use std::fs;

    #[derive(Debug, PartialEq, Serialize, Deserialize, Resource, Default)]
    struct DummySettings {
        volume: f32,
    }

    #[test]
    fn auto_timing_saves_on_change() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.ron");
        let mut app = App::new();
        app.add_plugins(SavePlugin);
        app.register_saved_resource::<DummySettings>(
            SaveLocation::Custom(path.clone()),
            SaveTiming::Auto,
        );

        app.update();

        app.world_mut().resource_mut::<DummySettings>().volume = 0.8;
        app.update();

        let saved = fs::read_to_string(&path).expect("file should exist after auto-save");
        assert!(saved.contains("0.8"));
    }

    #[test]
    fn manual_timing_does_not_save_on_change() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.ron");
        let mut app = App::new();
        app.add_plugins(SavePlugin);
        app.register_saved_resource::<DummySettings>(
            SaveLocation::Custom(path.clone()),
            SaveTiming::Manual,
        );

        app.world_mut().resource_mut::<DummySettings>().volume = 0.8;
        app.update();
        assert!(!path.exists(), "manual timing must not write on its own");

        save_now::<DummySettings>(app.world()).expect("manual save should succeed");
        assert!(path.exists());
    }

    #[test]
    fn register_loads_existing_save_file_on_startup() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.ron");
        fs::write(&path, "(volume: 0.42)").unwrap();

        let mut app = App::new();
        app.add_plugins(SavePlugin);
        app.register_saved_resource::<DummySettings>(
            SaveLocation::Custom(path),
            SaveTiming::Manual,
        );

        let resource = app.world().resource::<DummySettings>();
        assert_eq!(resource.volume, 0.42);
    }

    #[test]
    fn register_falls_back_to_default_when_no_save_file_exists() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("does_not_exist.ron");

        let mut app = App::new();
        app.add_plugins(SavePlugin);
        app.register_saved_resource::<DummySettings>(
            SaveLocation::Custom(path),
            SaveTiming::Manual,
        );

        let resource = app.world().resource::<DummySettings>();
        assert_eq!(*resource, DummySettings::default());
    }

    #[test]
    fn repeated_auto_save_registration_preserves_latest_path() {
        let dir = tempfile::tempdir().unwrap();
        let path1 = dir.path().join("first.ron");
        let path2 = dir.path().join("second.ron");
        let mut app = App::new();
        app.add_plugins(SavePlugin);
        app.register_saved_resource::<DummySettings>(
            SaveLocation::Custom(path1.clone()),
            SaveTiming::Auto,
        );
        app.register_saved_resource::<DummySettings>(
            SaveLocation::Custom(path2.clone()),
            SaveTiming::Auto,
        );

        app.update();

        app.world_mut().resource_mut::<DummySettings>().volume = 0.8;
        app.update();

        assert!(!path1.exists(), "first path must not be written");
        let saved = fs::read_to_string(&path2).expect("second path should exist after auto-save");
        assert!(saved.contains("0.8"));
    }

    #[test]
    fn repeated_auto_save_registration_installs_system_only_once() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("save.ron");

        let mut app = App::new();
        app.add_plugins(SavePlugin);
        app.register_saved_resource::<DummySettings>(
            SaveLocation::Custom(path.clone()),
            SaveTiming::Auto,
        );
        app.register_saved_resource::<DummySettings>(
            SaveLocation::Custom(path.clone()),
            SaveTiming::Auto,
        );

        app.update();
        fs::create_dir(&path).unwrap();

        app.world_mut().resource_mut::<DummySettings>().volume = 0.8;
        app.update();

        let messages = app.world().resource::<Messages<SaveFailed>>();
        assert_eq!(
            messages.len(),
            1,
            "auto_save_system should execute only once"
        );
    }

    #[test]
    #[should_panic(expected = "SavePlugin` must be added")]
    fn register_panics_without_save_plugin() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.ron");

        let mut app = App::new();
        app.register_saved_resource::<DummySettings>(
            SaveLocation::Custom(path),
            SaveTiming::Manual,
        );
    }
}
