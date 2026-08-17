use ron::Error as RonError;
use ron::error::SpannedError;
use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::Error as IoError;
use std::path::PathBuf;
use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum SaveWriteError {
    Io {
        path: PathBuf,
        source: IoError,
    },
    Serialize(RonError),
    ResourceMissing(String),
    UnknownGroup(String),
    /// Should not normally occur.
    Internal(String),
}

#[derive(Debug)]
pub enum SaveReadError {
    Io { path: PathBuf, source: IoError },
    Deserialize(SpannedError),
    InvalidUtf8(FromUtf8Error),
    GroupMemberDeserialize(RonError),
    UnknownGroup(String),
}

impl Display for SaveWriteError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            SaveWriteError::Io { path, source } => {
                write!(f, "io error at {}: {}", path.display(), source)
            }
            SaveWriteError::Serialize(e) => write!(f, "failed to serialize to RON: {e}"),
            SaveWriteError::ResourceMissing(type_name) => {
                write!(f, "resource `{type_name}` was not found in the world")
            }
            SaveWriteError::UnknownGroup(name) => write!(f, "unknown save group: {name}"),
            SaveWriteError::Internal(reason) => {
                write!(f, "internal error while saving: {reason}")
            }
        }
    }
}

impl Display for SaveReadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            SaveReadError::Io { path, source } => {
                write!(f, "io error at {}: {}", path.display(), source)
            }
            SaveReadError::Deserialize(e) => write!(f, "failed to deserialize from RON: {e}"),
            SaveReadError::InvalidUtf8(e) => write!(f, "saved file is not valid UTF-8: {e}"),
            SaveReadError::GroupMemberDeserialize(e) => {
                write!(f, "failed to deserialize group member from RON: {e}")
            }
            SaveReadError::UnknownGroup(name) => write!(f, "unknown save group: {name}"),
        }
    }
}

impl StdError for SaveWriteError {}
impl StdError for SaveReadError {}
