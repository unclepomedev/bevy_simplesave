use crate::error::SaveError;
use ron::ser::{PrettyConfig, to_string_pretty};
use serde::{Serialize, de::DeserializeOwned};

pub fn serialize_to_ron<R: Serialize>(value: &R) -> Result<String, SaveError> {
    to_string_pretty(value, PrettyConfig::default()).map_err(SaveError::Serialize)
}

pub fn deserialize_from_ron<R: DeserializeOwned>(s: &str) -> Result<R, SaveError> {
    ron::from_str(s).map_err(SaveError::Deserialize)
}
