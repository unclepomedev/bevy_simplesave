use crate::SaveError;
use crate::plugin::SavePath;
use crate::resource::save_resource;
use bevy_ecs::prelude::{Resource, World};
use serde::Serialize;
use std::any::type_name;

/// Explicitly saves a resource that was registered with [`SaveTiming::Manual`]
/// or force-saves one registered with [`SaveTiming::Auto`].
pub fn save_now<R: Resource + Serialize>(world: &World) -> bevy_ecs::error::Result<(), SaveError> {
    let path = world
        .get_resource::<SavePath<R>>()
        .ok_or_else(|| SaveError::ResourceMissing(type_name::<SavePath<R>>().to_string()))?;
    save_resource::<R>(world, &path.path_buf.clone())
}
