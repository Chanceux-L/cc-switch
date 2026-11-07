//! Qoder 的 MCP 配置存放在 `~/.qoder/settings.json`。Qoder 只支持 MCP 管理，
//! 不支持供应商切换，因此这里只提供数据目录与「先写文件后入库」的提交助手。
use crate::config::{atomic_write_private, get_home_dir};
use crate::error::AppError;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) fn data_dir() -> PathBuf {
    explicit_data_dir(std::env::var("QODER_CONFIG_DIR").ok().as_deref())
        .unwrap_or_else(|| get_home_dir().join(".qoder"))
}

fn explicit_data_dir(qoder: Option<&str>) -> Option<PathBuf> {
    qoder
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
}

// Callers hold their feature lock across the native write and database commit.
pub(crate) fn write_and_commit<T>(
    path: &Path,
    write: impl FnOnce() -> Result<(), AppError>,
    commit: impl FnOnce() -> Result<T, AppError>,
) -> Result<T, AppError> {
    let previous = match fs::read(path) {
        Ok(bytes) => Some(bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(AppError::io(path, error)),
    };
    write()?;
    match commit() {
        Ok(result) => Ok(result),
        Err(error) => {
            let rollback = match previous {
                Some(bytes) => atomic_write_private(path, &bytes),
                None => fs::remove_file(path).map_err(|error| AppError::io(path, error)),
            };
            if let Err(rollback_error) = rollback {
                return Err(AppError::Message(format!(
                    "Config update failed ({error}); restoring {} also failed: {rollback_error}",
                    path.display()
                )));
            }
            Err(error)
        }
    }
}

pub(crate) fn settings_path() -> PathBuf {
    data_dir().join("settings.json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn explicit_data_directory_resolves_and_ignores_blank_values() {
        assert_eq!(
            explicit_data_dir(Some(" /primary ")),
            Some("/primary".into())
        );
        assert_eq!(explicit_data_dir(Some(" \t")), None);
        assert_eq!(explicit_data_dir(None), None);
    }

    #[test]
    fn rollback_restores_previous_file_and_removal_on_commit_failure() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        fs::write(&path, json!({"enabledPlugins": {"keep": true}}).to_string()).unwrap();

        let committed: Result<(), AppError> = write_and_commit(
            &path,
            || Ok(()),
            || Err(AppError::Config("commit failed".into())),
        );
        assert!(committed.is_err());
        let value: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(value["enabledPlugins"]["keep"], json!(true));

        fs::remove_file(&path).unwrap();
        let result: Result<(), AppError> = write_and_commit(
            &path,
            || Ok(()),
            || Err(AppError::Config("commit failed".into())),
        );
        result.unwrap_err();
        assert!(!path.exists());
    }
}
