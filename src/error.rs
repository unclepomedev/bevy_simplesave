use ron::{Error as RonError, error::SpannedError};
use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::Error as IoError;
use std::path::PathBuf;

#[derive(Debug)]
pub enum SaveError {
    Io { path: PathBuf, source: IoError },
    Serialize(RonError),
    Deserialize(SpannedError),
    LocationUnavailable(String),
}

impl Display for SaveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            SaveError::Io { path, source } => {
                write!(f, "io error at {}: {}", path.display(), source)
            }
            SaveError::Serialize(e) => write!(f, "failed to serialize to RON: {e}"),
            SaveError::Deserialize(e) => write!(f, "failed to deserialize from RON: {e}"),
            SaveError::LocationUnavailable(reason) => {
                write!(f, "could not resolve save location: {reason}")
            }
        }
    }
}

impl StdError for SaveError {}
