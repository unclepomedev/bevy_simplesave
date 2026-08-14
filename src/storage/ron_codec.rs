use crate::error::SaveError;
use ron::ser::{PrettyConfig, to_string_pretty};
use serde::{Serialize, de::DeserializeOwned};

#[cfg_attr(not(test), expect(dead_code))]
pub(crate) fn serialize_to_ron<R: Serialize>(value: &R) -> Result<String, SaveError> {
    to_string_pretty(value, PrettyConfig::default()).map_err(SaveError::Serialize)
}

#[cfg_attr(not(test), expect(dead_code))]
pub(crate) fn deserialize_from_ron<R: DeserializeOwned>(s: &str) -> Result<R, SaveError> {
    ron::from_str(s).map_err(SaveError::Deserialize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct DummySettings {
        volume: f32,
        difficulty: u8,
    }

    #[test]
    fn ron_string_round_trip() {
        let original = DummySettings {
            volume: 0.8,
            difficulty: 2,
        };

        let ron_str = serialize_to_ron(&original).expect("serialize should succeed");
        let restored: DummySettings =
            deserialize_from_ron(&ron_str).expect("deserialize should succeed");

        assert_eq!(original, restored);
    }

    #[test]
    fn deserialize_invalid_ron_returns_err() {
        let result: Result<DummySettings, _> = deserialize_from_ron("not valid ron {{{");
        assert!(result.is_err());
    }
}
