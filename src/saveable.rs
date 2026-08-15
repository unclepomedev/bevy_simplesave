use crate::SaveTiming;
use bevy_ecs::prelude::Resource;
use serde::{Serialize, de::DeserializeOwned};

/// Resources that carry their own [`SaveTiming`], typically implemented via `#[derive(SaveResource)]`.
pub trait Saveable: Resource + Serialize + DeserializeOwned + Default {
    const TIMING: SaveTiming;
}
