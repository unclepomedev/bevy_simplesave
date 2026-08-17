use crate::error::SaveError;
use crate::{LoadFailed, storage};
use bevy_ecs::resource::Resource;
use bevy_ecs::world::World;
use ron::Value as RonValue;
use serde::{Serialize, de::DeserializeOwned};
use std::any::type_name;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::marker::PhantomData;
use std::path::Path;

pub(crate) type SaveBag = HashMap<String, RonValue>;

/// A type-erased save/load operation for one resource type within a group.
pub(crate) trait ErasedSave: Send + Sync + 'static {
    fn type_key(&self) -> &'static str;
    fn extract(&self, world: &World) -> Result<RonValue, SaveError>;
    fn apply(&self, world: &mut World, value: Option<RonValue>);
}

pub(crate) struct SaveEntry<R>(PhantomData<fn() -> R>);

impl<R> SaveEntry<R> {
    pub(crate) fn new() -> Self {
        Self(PhantomData)
    }
}

impl<R> ErasedSave for SaveEntry<R>
where
    R: Resource + Serialize + DeserializeOwned + Default,
{
    fn type_key(&self) -> &'static str {
        type_name::<R>()
    }

    fn extract(&self, world: &World) -> Result<RonValue, SaveError> {
        let resource = world
            .get_resource::<R>()
            .ok_or_else(|| SaveError::ResourceMissing(type_name::<R>().to_string()))?;
        let ron_str = storage::serialize_to_ron(resource)?;
        storage::deserialize_from_ron::<RonValue>(&ron_str)
    }

    /// - Key absent from the file falls back to `Default`.
    /// - Value that fails to deserialize falls back to `Default` and emits a `LoadFailed` message.
    fn apply(&self, world: &mut World, value: Option<RonValue>) {
        match value {
            Some(v) => match R::deserialize(v) {
                Ok(val) => {
                    world.insert_resource(val);
                }
                Err(e) => {
                    world.insert_resource(R::default());
                    world.write_message(LoadFailed {
                        resource_type: type_name::<R>(),
                        error: SaveError::GroupMemberDeserialize(e),
                    });
                }
            },
            None => {
                world.insert_resource(R::default());
            }
        }
    }
}

pub(crate) fn save_group_bag(
    world: &World,
    entries: &[Box<dyn ErasedSave>],
    path: &Path,
) -> Result<(), SaveError> {
    let mut bag = SaveBag::new();
    for entry in entries {
        bag.insert(entry.type_key().to_string(), entry.extract(world)?);
    }
    let ron_str = storage::serialize_to_ron(&bag)?;
    storage::write_bytes(path, ron_str.as_bytes())
}

pub(crate) fn load_group_bag(
    world: &mut World,
    entries: &[Box<dyn ErasedSave>],
    path: &Path,
) -> Result<(), SaveError> {
    let bytes = match storage::read_bytes(path) {
        Ok(bytes) => bytes,
        Err(SaveError::Io { source, .. }) if source.kind() == ErrorKind::NotFound => {
            for entry in entries {
                entry.apply(world, None);
            }
            return Ok(());
        }
        Err(e) => return Err(e),
    };

    let ron_str = String::from_utf8(bytes).map_err(SaveError::InvalidUtf8)?;
    let mut bag: SaveBag = storage::deserialize_from_ron(&ron_str)?;

    for entry in entries {
        let value = bag.remove(entry.type_key());
        entry.apply(world, value);
    }
    Ok(())
}
