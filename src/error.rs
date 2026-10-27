use ron::{Error as RonError, error::SpannedError};
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    io::{Error as IoError, ErrorKind},
    path::PathBuf,
    string::FromUtf8Error,
};

#[derive(Debug)]
pub enum SaveWriteError {
    Io {
        path: PathBuf,
        source: IoError,
    },
    Serialize {
        resource_type: String,
        source: RonError,
    },
    ResourceMissing {
        resource_type: String,
    },
    UnknownGroup {
        group: String,
    },
    /// Should not normally occur.
    Internal(String),
}

#[derive(Debug)]
pub enum SaveReadError {
    Io {
        path: PathBuf,
        source: IoError,
    },
    Deserialize(SpannedError),
    InvalidUtf8(FromUtf8Error),
    GroupMemberDeserialize {
        resource_type: String,
        source: Box<SpannedError>,
    },
    MissingMember {
        resource_type: String,
    },
    UnknownGroup {
        group: String,
    },
}

impl Display for SaveWriteError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            SaveWriteError::Io { path, source } => {
                write!(f, "io error at {}: {}", path.display(), source)
            }
            SaveWriteError::Serialize {
                resource_type,
                source,
            } => write!(f, "failed to serialize `{resource_type}` to RON: {source}"),
            SaveWriteError::ResourceMissing { resource_type } => {
                write!(f, "resource `{resource_type}` was not found in the world")
            }
            SaveWriteError::UnknownGroup { group } => write!(f, "unknown save group: {group}"),
            SaveWriteError::Internal(reason) => {
                write!(
                    f,
                    "internal error while saving (this is likely to be a bug in bevy_simplesave): {reason}"
                )
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
            SaveReadError::GroupMemberDeserialize {
                resource_type,
                source,
            } => {
                write!(
                    f,
                    "failed to deserialize `{resource_type}` from RON: {source}"
                )
            }
            SaveReadError::MissingMember { resource_type } => {
                write!(f, "save data has no entry for `{resource_type}`")
            }
            SaveReadError::UnknownGroup { group: name } => write!(f, "unknown save group: {name}"),
        }
    }
}

impl StdError for SaveWriteError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            SaveWriteError::Io { source, .. } => Some(source),
            SaveWriteError::Serialize { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl StdError for SaveReadError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            SaveReadError::Io { source, .. } => Some(source),
            SaveReadError::Deserialize(source) => Some(source),
            SaveReadError::InvalidUtf8(source) => Some(source),
            SaveReadError::GroupMemberDeserialize { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl SaveReadError {
    pub fn is_not_found(&self) -> bool {
        matches!(self, SaveReadError::Io { source, .. } if source.kind() == ErrorKind::NotFound)
    }
}
