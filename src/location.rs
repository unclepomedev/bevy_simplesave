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
    #[cfg_attr(not(test), expect(dead_code))]
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

// ============================================================================================
// UNIT TESTS
// ============================================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_returns_path_as_is() {
        let location = SaveLocation::Custom(PathBuf::from("/tmp/my-game/save.ron"));
        let resolved = location
            .resolve()
            .expect("custom path should always resolve");
        assert_eq!(resolved, PathBuf::from("/tmp/my-game/save.ron"));
    }

    #[test]
    fn exe_relative_joins_sub_path_to_exe_dir() {
        let location = SaveLocation::ExeRelative(PathBuf::from("saves/settings.ron"));
        let resolved = location
            .resolve()
            .expect("current_exe should be available in tests");

        let expected_exe_dir = env::current_exe().unwrap().parent().unwrap().to_path_buf();
        assert_eq!(resolved, expected_exe_dir.join("saves/settings.ron"));
    }

    #[test]
    fn app_data_includes_application_name_in_path() {
        let location = SaveLocation::AppData {
            qualifier: "dev",
            organization: "ExampleStudio",
            application: "ExampleGame",
            sub: PathBuf::from("settings.ron"),
        };
        let resolved = location.resolve().expect("should resolve on CI runners");

        let resolved_str = resolved.to_string_lossy().to_lowercase();
        assert!(
            resolved_str.contains("examplegame"),
            "resolved path `{resolved_str}` should include the application name"
        );
        assert!(resolved_str.ends_with("settings.ron"));
    }
}
