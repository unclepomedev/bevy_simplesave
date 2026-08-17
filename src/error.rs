use ron::{Error as RonError, error::SpannedError};
use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::Error as IoError;
use std::path::PathBuf;
use std::string::FromUtf8Error;

// TODO: split / organize
#[derive(Debug)]
pub enum SaveError {
    Io { path: PathBuf, source: IoError },
    Serialize(RonError),
    Deserialize(SpannedError),
    LocationUnavailable(String),
    InvalidSubPath(PathBuf),
    ResourceMissing(String),
    InvalidUtf8(FromUtf8Error),
    GroupMemberDeserialize(RonError),
    UnknownGroup(String),
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
            SaveError::InvalidSubPath(path) => {
                write!(
                    f,
                    "sub path `{}` must be relative (no root or drive prefix)",
                    path.display()
                )
            }
            SaveError::ResourceMissing(type_name) => {
                write!(f, "resource `{type_name}` was not found in the world")
            }
            SaveError::InvalidUtf8(e) => {
                write!(f, "saved file is not valid UTF-8: {e}")
            }
            SaveError::GroupMemberDeserialize(e) => {
                write!(f, "failed to deserialize group member from RON: {e}")
            }
            SaveError::UnknownGroup(name) => {
                write!(f, "unknown save group: {name}")
            }
        }
    }
}

impl StdError for SaveError {}
