use crate::{SaveWriteError, plugin::SavePath, resource::save_resource};
use bevy_ecs::prelude::*;
use serde::Serialize;
use std::any::type_name;

/// Explicitly saves a resource registered with [`SaveTiming::Manual`]
/// or force-saves one registered with [`SaveTiming::Auto`].
pub fn save_now<R: Resource + Serialize>(world: &World) -> Result<(), SaveWriteError> {
    let path =
        world
            .get_resource::<SavePath<R>>()
            .ok_or_else(|| SaveWriteError::ResourceMissing {
                resource_type: type_name::<SavePath<R>>().to_string(),
            })?;
    save_resource::<R>(world, &path.path_buf.clone())
}
