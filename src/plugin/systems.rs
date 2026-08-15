use crate::plugin::save_path::SavePath;
use crate::resource::write_resource_ron;
use bevy_ecs::prelude::*;
use serde::Serialize;
use std::any::type_name;

pub(crate) fn auto_save_system<R: Resource + Serialize>(resource: Res<R>, path: Res<SavePath<R>>) {
    if resource.is_changed()
        && !resource.is_added()
        && let Err(e) = write_resource_ron(&*resource, &path.path_buf)
    {
        // TODO: reconsider panic vs. a `SaveFailed` Message.
        eprintln!(
            "bevy_simplesave: failed to auto-save `{}`: {e}",
            type_name::<R>()
        );
    }
}
