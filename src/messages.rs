use crate::SaveError;
use bevy_ecs::prelude::*;

#[derive(Message, Debug)]
pub struct SaveFailed {
    pub resource_type: &'static str,
    pub error: SaveError,
}
