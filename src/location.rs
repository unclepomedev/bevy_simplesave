use crate::error::SaveError;
use directories::ProjectDirs;
use std::env;
use std::path::PathBuf;

/// Where a saved resource's `.ron` file should live on disk.
#[derive(Debug, Clone)]
pub enum SaveLocation {
    /// Relative to the directory containing the running executable.
    ExeRelative(PathBuf),
    /// The OS-standard application data directory
    /// (e.g. `~/.local/share/<app>` on Linux, `%APPDATA%\<org>\<app>` on Windows).
    AppData {
        qualifier: &'static str,
        organization: &'static str,
        application: &'static str,
        sub: PathBuf,
    },
    /// Any path chosen by the caller, used as-is.
    Custom(PathBuf),
}

impl SaveLocation {
    pub(crate) fn resolve(&self) -> Result<PathBuf, SaveError> {
        match self {
            SaveLocation::ExeRelative(sub) => {
                let exe = env::current_exe().map_err(|source| SaveError::Io {
                    path: PathBuf::from("<current_exe>"),
                    source,
                })?;
                let exe_dir = exe.parent().ok_or_else(|| {
                    SaveError::LocationUnavailable("executable path has no parent directory".into())
                })?;
                Ok(exe_dir.join(sub))
            }
            SaveLocation::AppData {
                qualifier,
                organization,
                application,
                sub,
            } => {
                let dirs =
                    ProjectDirs::from(qualifier, organization, application).ok_or_else(|| {
                        SaveError::LocationUnavailable(
                            "could not determine a home directory on this platform".into(),
                        )
                    })?;
                Ok(dirs.data_dir().join(sub))
            }
            SaveLocation::Custom(path) => Ok(path.clone()),
        }
    }
}
