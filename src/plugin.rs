mod app_ext;
mod save_path;
mod systems;
mod timing;

pub use self::app_ext::{SaveAppExt, save_now};
pub(crate) use self::save_path::SavePath;
pub use self::timing::SaveTiming;
use bevy_app::prelude::*;

/// Adds save/load support to the app. Currently a marker plugin; register
/// individual resources with [`SaveAppExt::register_saved_resource`].
#[derive(Default)]
pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, _app: &mut App) {
        // Intentionally empty for now. Resources are registered individually
        // via `SaveAppExt::register_saved_resource`, which does not depend on
        // this plugin having been added. This exists as a stable extension
        // point and to match the conventional `add_plugins(...)` pattern.
    }
}

// ============================================================================================
// UNIT TESTS
// ============================================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::SaveLocation;
    use crate::plugin::app_ext::{SaveAppExt, save_now};
    use bevy_ecs::resource::Resource;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize, Resource, Default)]
    struct DummySettings {
        volume: f32,
    }

    #[test]
    fn auto_timing_saves_on_change() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.ron");
        let mut app = App::new();
        app.register_saved_resource::<DummySettings>(
            SaveLocation::Custom(path.clone()),
            SaveTiming::Auto,
        );

        app.update();

        app.world_mut().resource_mut::<DummySettings>().volume = 0.8;
        app.update();

        let saved = std::fs::read_to_string(&path).expect("file should exist after auto-save");
        assert!(saved.contains("0.8"));
    }

    #[test]
    fn manual_timing_does_not_save_on_change() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.ron");
        let mut app = App::new();
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
        std::fs::write(&path, "(volume: 0.42)").unwrap();

        let mut app = App::new();
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
        app.register_saved_resource::<DummySettings>(
            SaveLocation::Custom(path),
            SaveTiming::Manual,
        );

        let resource = app.world().resource::<DummySettings>();
        assert_eq!(*resource, DummySettings::default());
    }
}
