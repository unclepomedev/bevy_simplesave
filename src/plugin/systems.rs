use crate::messages::SaveFailed;
use crate::plugin::save_path::SavePath;
use crate::resource::write_resource_ron;
use bevy_ecs::prelude::*;
use serde::Serialize;
use std::any::type_name;

pub(crate) fn auto_save_system<R: Resource + Serialize>(
    resource: Res<R>,
    path: Res<SavePath<R>>,
    mut failures: MessageWriter<SaveFailed>,
) {
    if resource.is_changed()
        && !resource.is_added()
        && let Err(e) = write_resource_ron(&*resource, &path.path_buf)
    {
        failures.write(SaveFailed {
            resource_type: type_name::<R>(),
            error: e,
        });
    }
}
