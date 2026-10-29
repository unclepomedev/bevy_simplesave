mod error;
mod group;
mod group_ext;
mod location;
mod storage;

pub use crate::{
    error::{SaveReadError, SaveWriteError},
    group::{SaveGroup, SaveMember, read_member},
    group_ext::{SaveGroupExt, load_group, save_group},
    location::{LocationError, SaveLocation},
};

/// Derives [`SaveMember`], reading the key from `#[save(key = "...")]`.
///
/// The key must be unique within a group and must not change once files exist.
///
/// # Example
/// ```
/// use bevy_ecs::prelude::Resource;
/// use bevy_simplesave::SaveMember;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Resource, Serialize, Deserialize, SaveMember)]
/// #[save(key = "position")]
/// struct Position {
///     x: f32,
/// }
///
/// assert_eq!(<Position as SaveMember>::KEY, "position");
/// ```
pub use bevy_simplesave_derive::SaveMember;
