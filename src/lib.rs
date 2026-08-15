mod error;
mod location;
mod messages;
mod plugin;
mod resource;
mod saveable;
mod storage;

pub use error::SaveError;
pub use location::SaveLocation;
pub use messages::SaveFailed;
pub use plugin::{SaveAppExt, SavePlugin, SaveTiming, save_now};
pub use saveable::Saveable;

/// Derives [`Saveable`] for a resource, reading the save timing from
/// `#[save(timing = auto | manual)]`.
///
/// # Example
/// ```
/// use bevy_app::App;
/// use bevy_ecs::prelude::Resource;
/// use bevy_simplesave::{SaveAppExt, SaveLocation, SavePlugin, SaveResource};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Resource, Serialize, Deserialize, Default, SaveResource)]
/// #[save(timing = manual)]
/// struct Settings {
///     volume: f32,
/// }
///
/// let mut app = App::new();
/// app.add_plugins(SavePlugin);
/// app.register_saveable::<Settings>(SaveLocation::Custom("/tmp/example_settings.ron".into()));
/// ```
pub use bevy_simplesave_derive::SaveResource;
