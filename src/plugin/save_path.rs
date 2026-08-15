use bevy_ecs::prelude::Resource;
use std::marker::PhantomData;
use std::path::PathBuf;

#[derive(Resource)]
pub(crate) struct SavePath<R> {
    pub(crate) path_buf: PathBuf,
    pub(crate) phantom_data: PhantomData<fn() -> R>,
}
