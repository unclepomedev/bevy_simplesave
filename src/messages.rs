use crate::{SaveReadError, SaveWriteError};
use bevy_ecs::prelude::*;

#[derive(Message, Debug)]
pub struct SaveFailed {
    pub resource_type: &'static str,
    pub error: SaveWriteError,
}

#[derive(Message, Debug)]
pub struct LoadFailed {
    pub resource_type: &'static str,
    pub error: SaveReadError,
}
