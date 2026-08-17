use crate::storage::StorageError;
use crate::{LoadFailed, SaveReadError, SaveWriteError, storage};
use bevy_ecs::resource::Resource;
use bevy_ecs::world::World;
use ron::Value as RonValue;
use serde::{Serialize, de::DeserializeOwned};
use std::any::type_name;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::marker::PhantomData;
use std::path::Path;

/// A marker type identifying a save group. Implement this on an empty
/// struct to give the group a name at the type level.
///
/// ```
/// use bevy_simplesave::SaveGroup;
///
/// struct SlotGroup;
/// impl SaveGroup for SlotGroup {}
/// ```
pub trait SaveGroup: 'static {}

pub(crate) type SaveBag = HashMap<String, RonValue>;

/// A type-erased save/load operation for one resource type within a group.
pub(crate) trait ErasedSave: Send + Sync + 'static {
    fn type_key(&self) -> &'static str;
    fn extract(&self, world: &World) -> Result<RonValue, SaveWriteError>;
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

    fn extract(&self, world: &World) -> Result<RonValue, SaveWriteError> {
        let resource =
            world
                .get_resource::<R>()
                .ok_or_else(|| SaveWriteError::ResourceMissing {
                    resource_type: type_name::<R>().to_string(),
                })?;
        let ron_str = storage::serialize_to_ron(resource).map_err(|e| match e {
            StorageError::Serialize(e) => SaveWriteError::Serialize {
                resource_type: type_name::<R>().to_string(),
                source: e,
            },
            other => SaveWriteError::Internal(format!("{}: {other}", type_name::<R>())),
        })?;
        storage::deserialize_from_ron::<RonValue>(&ron_str).map_err(|e| {
            SaveWriteError::Internal(format!(
                "re-parsing RON produced for `{}` failed: {e}",
                type_name::<R>()
            ))
        })
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
                        error: SaveReadError::GroupMemberDeserialize(e),
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
) -> Result<(), SaveWriteError> {
    let mut bag = SaveBag::new();
    for entry in entries {
        bag.insert(entry.type_key().to_string(), entry.extract(world)?);
    }
    let ron_str = storage::serialize_to_ron(&bag).map_err(|e| match e {
        StorageError::Serialize(e) => SaveWriteError::Serialize {
            resource_type: "SaveBag".to_string(),
            source: e,
        },
        other => SaveWriteError::Internal(other.to_string()),
    })?;
    storage::write_bytes(path, ron_str.as_bytes()).map_err(|e| match e {
        StorageError::Io { path, source } => SaveWriteError::Io { path, source },
        other => SaveWriteError::Internal(other.to_string()),
    })
}

pub(crate) fn load_group_bag(
    world: &mut World,
    entries: &[Box<dyn ErasedSave>],
    path: &Path,
) -> Result<(), SaveReadError> {
    let bytes = match storage::read_bytes(path) {
        Ok(bytes) => bytes,
        Err(StorageError::Io { source, .. }) if source.kind() == ErrorKind::NotFound => {
            for entry in entries {
                entry.apply(world, None);
            }
            return Ok(());
        }
        Err(e) => {
            return Err(match e {
                StorageError::Io { path, source } => SaveReadError::Io { path, source },
                _other => unreachable!("read_bytes only returns Io errors: {_other}"),
            });
        }
    };

    let ron_str = String::from_utf8(bytes).map_err(SaveReadError::InvalidUtf8)?;
    let mut bag: SaveBag = storage::deserialize_from_ron(&ron_str).map_err(|e| match e {
        StorageError::Deserialize(e) => SaveReadError::Deserialize(e),
        _other => unreachable!("deserialize_from_ron only returns Deserialize errors: {_other}"),
    })?;

    for entry in entries {
        let value = bag.remove(entry.type_key());
        entry.apply(world, value);
    }
    Ok(())
}
