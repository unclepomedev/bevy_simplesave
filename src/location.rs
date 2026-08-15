use crate::error::SaveError;
use directories::ProjectDirs;
use std::env;
use std::path::{Component, Path, PathBuf};

/// Where a saved resource's `.ron` file should live on disk.
#[derive(Debug, Clone)]
pub enum SaveLocation {
    /// Relative to the directory containing the running executable.
    ExeRelative(PathBuf),
    /// The OS-standard application data directory
    /// (e.g. `~/.local/share/<app>` on Linux, `%APPDATA%\<org>\<app>\data` on Windows).
    AppData {
        qualifier: &'static str,
        organization: &'static str,
        application: &'static str,
        sub: PathBuf,
    },
    /// Any path chosen by the caller, used as-is.
    Custom(PathBuf),
}

fn validate_sub_path(sub: &Path) -> Result<(), SaveError> {
    if sub.components().any(|c| {
        matches!(
            c,
            Component::Prefix(_) | Component::RootDir | Component::ParentDir
        )
    }) {
        return Err(SaveError::InvalidSubPath(sub.to_path_buf()));
    }
    Ok(())
}

fn resolve_app_data_dir(dirs: Option<ProjectDirs>, sub: &Path) -> Result<PathBuf, SaveError> {
    validate_sub_path(sub)?;
    let dirs = dirs.ok_or_else(|| {
        SaveError::LocationUnavailable(
            "could not determine a home directory on this platform".into(),
        )
    })?;
    Ok(dirs.data_dir().join(sub))
}

impl SaveLocation {
    #[cfg_attr(not(test), expect(dead_code))]
    pub(crate) fn resolve(&self) -> Result<PathBuf, SaveError> {
        match self {
            SaveLocation::ExeRelative(sub) => {
                validate_sub_path(sub)?;
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
                let dirs = ProjectDirs::from(qualifier, organization, application);
                resolve_app_data_dir(dirs, sub)
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
    use std::assert_matches;

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
    fn app_data_resolves_to_project_dirs_data_dir_when_available() {
        let dirs = ProjectDirs::from("dev", "ExampleStudio", "ExampleGame")
            .expect("this environment must have a resolvable home directory");
        let sub = PathBuf::from("settings.ron");

        let resolved = resolve_app_data_dir(Some(dirs.clone()), &sub)
            .expect("resolve must succeed when ProjectDirs is Some");

        assert_eq!(resolved, dirs.data_dir().join("settings.ron"));
    }

    #[test]
    fn app_data_returns_err_when_project_dirs_is_none() {
        let sub = PathBuf::from("settings.ron");
        let err = resolve_app_data_dir(None, &sub)
            .expect_err("resolve must fail when ProjectDirs is None");
        assert_matches!(err, SaveError::LocationUnavailable(_));
    }

    #[test]
    fn exe_relative_rejects_parent_dir() {
        let sub = PathBuf::from("../save.ron");
        let location = SaveLocation::ExeRelative(sub.clone());
        let err = location
            .resolve()
            .expect_err("parent dir subpath must fail");
        assert_matches!(err, SaveError::InvalidSubPath(p) if p == sub);
    }

    #[test]
    fn app_data_rejects_parent_dir() {
        let sub = PathBuf::from("../save.ron");
        let dirs = ProjectDirs::from("dev", "ExampleStudio", "ExampleGame");
        let err = resolve_app_data_dir(dirs, &sub).expect_err("parent dir subpath must fail");
        assert_matches!(err, SaveError::InvalidSubPath(p) if p == sub);
    }

    #[test]
    #[cfg(unix)]
    fn exe_relative_rejects_rooted_sub_path_unix() {
        let sub = PathBuf::from("/root/save.ron");
        let location = SaveLocation::ExeRelative(sub.clone());
        let err = location.resolve().expect_err("rooted subpath must fail");
        assert_matches!(err, SaveError::InvalidSubPath(p) if p == sub);
    }

    #[test]
    #[cfg(unix)]
    fn app_data_rejects_rooted_sub_path_unix() {
        let sub = PathBuf::from("/root/save.ron");
        let dirs = ProjectDirs::from("dev", "ExampleStudio", "ExampleGame");
        let err = resolve_app_data_dir(dirs, &sub).expect_err("rooted subpath must fail");
        assert_matches!(err, SaveError::InvalidSubPath(p) if p == sub);
    }

    #[test]
    #[cfg(windows)]
    fn exe_relative_rejects_rooted_sub_path_windows() {
        for invalid in [
            r"C:\saves\settings.ron",
            r"C:saves\settings.ron",
            r"\saves\settings.ron",
            r"\\server\share\settings.ron",
        ] {
            let sub = PathBuf::from(invalid);
            let location = SaveLocation::ExeRelative(sub.clone());
            let err = location
                .resolve()
                .expect_err("rooted/prefixed subpath must fail on Windows");
            assert_matches!(err, SaveError::InvalidSubPath(p) if p == sub);
        }
    }

    #[test]
    #[cfg(windows)]
    fn app_data_rejects_rooted_sub_path_windows() {
        for invalid in [
            r"C:\saves\settings.ron",
            r"C:saves\settings.ron",
            r"\saves\settings.ron",
            r"\\server\share\settings.ron",
        ] {
            let sub = PathBuf::from(invalid);
            let dirs = ProjectDirs::from("dev", "ExampleStudio", "ExampleGame");
            let err = resolve_app_data_dir(dirs, &sub)
                .expect_err("rooted/prefixed subpath must fail on Windows");
            assert_matches!(err, SaveError::InvalidSubPath(p) if p == sub);
        }
    }
}
