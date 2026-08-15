mod error;
mod location;
mod plugin;
mod resource;
mod storage;

pub use error::SaveError;
pub use location::SaveLocation;
pub use plugin::{SaveAppExt, SavePlugin, SaveTiming, save_now};
