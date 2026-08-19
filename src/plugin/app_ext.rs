use super::systems::auto_save_system;
use super::{SavePath, ensure_save_plugin_added};
use crate::resource::load_resource;
use crate::{LoadFailed, SaveLocation, SaveTiming, Saveable};
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use serde::{Serialize, de::DeserializeOwned};
use std::any::type_name;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

/// Marker resource used to guard against registering `auto_save_system::<R>`
/// more than once if `register_saved_resource::<R>` is called multiple times.
#[derive(Resource)]
struct AutoSaveRegistered<R>(PhantomData<fn() -> R>);

pub trait SaveAppExt {
    fn register_saved_resource<R>(
        &mut self,
        location: SaveLocation,
        timing: SaveTiming,
    ) -> &mut Self
    where
        R: Resource + Serialize + DeserializeOwned + Default;

    fn register_saveable<R: Saveable>(&mut self, location: SaveLocation) -> &mut Self {
        self.register_saved_resource::<R>(location, R::TIMING)
    }
}

impl SaveAppExt for App {
    /// Panics if [`SimpleSavePlugin`] has not been added to the app.
    fn register_saved_resource<R>(
        &mut self,
        location: SaveLocation,
        timing: SaveTiming,
    ) -> &mut Self
    where
        R: Resource + Serialize + DeserializeOwned + Default,
    {
        ensure_save_plugin_added(self.world());
        let path = resolve_or_panic::<R>(location);

        let world = self.world_mut();
        load_or_fallback::<R>(world, &path);
        world.insert_resource(SavePath::<R> {
            path_buf: path,
            phantom_data: PhantomData,
        });
        let needs_system = claim_auto_save_registration::<R>(world, timing);

        if needs_system {
            self.add_systems(PostUpdate, auto_save_system::<R>);
        }
        self
    }
}

fn resolve_or_panic<R>(location: SaveLocation) -> PathBuf {
    location.resolve().unwrap_or_else(|e| {
        panic!(
            "bevy_simplesave: failed to resolve save location for `{}`: {e}",
            type_name::<R>()
        )
    })
}

fn load_or_fallback<R: Resource + DeserializeOwned + Default>(world: &mut World, path: &Path) {
    match load_resource::<R>(world, path) {
        Ok(true) => {}
        Ok(false) => world.insert_resource(R::default()),
        Err(e) => {
            world.insert_resource(R::default());
            world.write_message(LoadFailed {
                resource_type: type_name::<R>(),
                error: e,
            });
        }
    }
}

fn claim_auto_save_registration<R: Resource>(world: &mut World, timing: SaveTiming) -> bool {
    let needed = timing == SaveTiming::Auto && !world.contains_resource::<AutoSaveRegistered<R>>();
    if needed {
        world.insert_resource(AutoSaveRegistered::<R>(PhantomData));
    }
    needed
}
