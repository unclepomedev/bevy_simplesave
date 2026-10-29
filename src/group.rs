use crate::{SaveReadError, SaveWriteError, storage, storage::StorageError};
use bevy_ecs::{resource::Resource, world::World};
use ron::value::RawValue;
use serde::{Serialize, de::DeserializeOwned};
use std::{
    any::{TypeId, type_name},
    collections::HashMap,
    marker::PhantomData,
    path::Path,
};

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

/// A resource that can be saved as a member of a [`SaveGroup`] file, under a fixed key.
///
/// The key ties the data in a file to a type: keep it unique within a group and do not change it once files exist.
pub trait SaveMember: Resource + Serialize + DeserializeOwned {
    const KEY: &'static str;
}

pub fn read_member<R: SaveMember>(path: impl AsRef<Path>) -> Result<R, SaveReadError> {
    let mut bag = read_bag(path.as_ref())?;
    decode_member::<R>(bag.remove(R::KEY))
}

pub(crate) type SaveBag = HashMap<String, Box<RawValue>>;
pub(crate) type Commit = Box<dyn FnOnce(&mut World) + Send>;

/// A type-erased save/load operation for one resource type within a group.
pub(crate) trait ErasedSave: Send + Sync + 'static {
    fn key(&self) -> &'static str;
    fn member_type(&self) -> TypeId;
    fn extract(&self, world: &World) -> Result<Box<RawValue>, SaveWriteError>;
    /// Decodes without touching the world. Missing key or broken value is an error.
    fn prepare(&self, value: Option<Box<RawValue>>) -> Result<Commit, SaveReadError>;
}

pub(crate) struct SaveEntry<R>(PhantomData<fn() -> R>);

impl<R> SaveEntry<R> {
    pub(crate) fn new() -> Self {
        Self(PhantomData)
    }
}

impl<R: SaveMember> ErasedSave for SaveEntry<R> {
    fn key(&self) -> &'static str {
        R::KEY
    }

    fn member_type(&self) -> TypeId {
        TypeId::of::<R>()
    }

    fn extract(&self, world: &World) -> Result<Box<RawValue>, SaveWriteError> {
        let resource =
            world
                .get_resource::<R>()
                .ok_or_else(|| SaveWriteError::ResourceMissing {
                    resource_type: type_name::<R>().to_string(),
                })?;
        RawValue::from_rust(resource).map_err(|source| SaveWriteError::Serialize {
            resource_type: type_name::<R>().to_string(),
            source,
        })
    }

    fn prepare(&self, value: Option<Box<RawValue>>) -> Result<Commit, SaveReadError> {
        let resource = decode_member::<R>(value)?;
        Ok(Box::new(move |world: &mut World| {
            world.insert_resource(resource);
        }))
    }
}

pub(crate) fn save_group_bag(
    world: &World,
    entries: &[Box<dyn ErasedSave>],
    path: &Path,
) -> Result<(), SaveWriteError> {
    let mut bag = SaveBag::new();
    for entry in entries {
        bag.insert(entry.key().to_string(), entry.extract(world)?);
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

/// Interprets all members first and applies them to the world only if all of them are valid.
pub(crate) fn load_group_bag(
    world: &mut World,
    entries: &[Box<dyn ErasedSave>],
    path: &Path,
) -> Result<(), SaveReadError> {
    let mut bag = read_bag(path)?;
    let commits = entries
        .iter()
        .map(|entry| entry.prepare(bag.remove(entry.key())))
        .collect::<Result<Vec<_>, _>>()?;
    for commit in commits {
        commit(world);
    }
    Ok(())
}

fn read_bag(path: &Path) -> Result<SaveBag, SaveReadError> {
    let bytes = storage::read_bytes(path).map_err(|e| match e {
        StorageError::Io { path, source } => SaveReadError::Io { path, source },
        _other => unreachable!("read_bytes only returns Io errors: {_other}"),
    })?;
    let ron_str = String::from_utf8(bytes).map_err(SaveReadError::InvalidUtf8)?;
    storage::deserialize_from_ron(&ron_str).map_err(|e| match e {
        StorageError::Deserialize(e) => SaveReadError::Deserialize(e),
        _other => unreachable!("deserialize_from_ron only returns Deserialize errors: {_other}"),
    })
}

fn decode_member<R: DeserializeOwned>(value: Option<Box<RawValue>>) -> Result<R, SaveReadError> {
    let raw = value.ok_or_else(|| SaveReadError::MissingMember {
        resource_type: type_name::<R>().to_string(),
    })?;
    raw.into_rust::<R>()
        .map_err(|source| SaveReadError::GroupMemberDeserialize {
            resource_type: type_name::<R>().to_string(),
            source: Box::new(source),
        })
}
