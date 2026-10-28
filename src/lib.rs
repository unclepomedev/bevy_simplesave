mod error;
mod group;
mod location;
mod messages;
mod plugin;
mod resource;
mod saveable;
mod storage;

pub use crate::{
    error::{SaveReadError, SaveWriteError},
    group::{SaveGroup, read_member},
    location::{LocationError, SaveLocation},
    messages::{LoadFailed, SaveFailed},
    plugin::{
        SaveAppExt, SaveGroupExt, SaveTiming, SimpleSavePlugin, load_group, save_group, save_now,
    },
    saveable::Saveable,
};

/// Derives [`Saveable`] for a resource, reading the save timing from
/// `#[save(timing = auto | manual)]`.
///
/// # Example
/// ```
/// use bevy_app::App;
/// use bevy_ecs::prelude::Resource;
/// use bevy_simplesave::{SaveAppExt, SaveLocation, SimpleSavePlugin, SaveResource};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Resource, Serialize, Deserialize, Default, SaveResource)]
/// #[save(timing = manual)]
/// struct Settings {
///     volume: f32,
/// }
///
/// let mut app = App::new();
/// app.add_plugins(SimpleSavePlugin);
/// app.register_saveable::<Settings>(SaveLocation::Custom("/tmp/example_settings.ron".into()));
/// ```
pub use bevy_simplesave_derive::SaveResource;

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
