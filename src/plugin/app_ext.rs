use super::SavePath;
use super::systems::auto_save_system;
use crate::resource::{load_resource, save_resource};
use crate::{SaveError, SaveLocation, SaveTiming};
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use serde::{Serialize, de::DeserializeOwned};
use std::any::type_name;
use std::marker::PhantomData;

/// Explicitly saves a resource that was registered with [`SaveTiming::Manual`]
/// or force-saves one registered with [`SaveTiming::Auto`].
pub fn save_now<R: Resource + Serialize>(world: &World) -> Result<(), SaveError> {
    let path = world
        .get_resource::<SavePath<R>>()
        .ok_or_else(|| SaveError::ResourceMissing(type_name::<SavePath<R>>().to_string()))?;
    save_resource::<R>(world, &path.path_buf.clone())
}

pub trait SaveAppExt {
    fn register_saved_resource<R>(
        &mut self,
        location: SaveLocation,
        timing: SaveTiming,
    ) -> &mut Self
    where
        R: Resource + Serialize + DeserializeOwned + Default;
}

impl SaveAppExt for App {
    fn register_saved_resource<R>(
        &mut self,
        location: SaveLocation,
        timing: SaveTiming,
    ) -> &mut Self
    where
        R: Resource + Serialize + DeserializeOwned + Default,
    {
        let path = location.resolve().unwrap_or_else(|e| {
            panic!(
                "bevy_simplesave: failed to resolve save location for `{}`: {e}",
                type_name::<R>()
            )
        });

        let world = self.world_mut();
        if world.get_resource::<R>().is_none() {
            world.insert_resource(R::default());
        }
        load_resource::<R>(world, &path).unwrap_or_else(|e| {
            panic!(
                "bevy_simplesave: failed to load saved `{}` from `{}`: {e}",
                type_name::<R>(),
                path.display()
            )
        });
        world.insert_resource(SavePath::<R> {
            path_buf: path,
            phantom_data: PhantomData,
        });

        if timing == SaveTiming::Auto {
            self.add_systems(PostUpdate, auto_save_system::<R>);
        }
        self
    }
}
