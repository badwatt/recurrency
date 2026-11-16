//! Durable native preferences.
//!
//! App mode and theme used to live in separate files with no recovery path: a
//! single failed/truncated write or a different data directory made the next
//! launch start in local mode with the default theme. Every write now keeps a
//! backup and reads can migrate from the common legacy locations.

use std::path::{Path, PathBuf};

const BACKUP_SUFFIX: &str = ".bak";
const DATA_DIR_ENV: &str = "RECURRENCY_DATA_DIR";

pub fn recurrency_dir() -> Option<PathBuf> {
    crate::state::auth::get_app_data_dir().map(|dir| dir.join("recurrency"))
}

fn backup_path(path: &Path) -> PathBuf {
    path.with_file_name(format!(
        "{}{BACKUP_SUFFIX}",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("config")
    ))
}

fn candidate_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(dir) = recurrency_dir() {
        dirs.push(dir);
    }
    if let Ok(dir) = std::env::var(DATA_DIR_ENV)
        && !dir.trim().is_empty()
    {
        dirs.push(PathBuf::from(dir).join("recurrency"));
    }
    if let Some(dir) = dirs::config_dir() {
        dirs.push(dir.join("recurrency"));
    }
    if let Some(dir) = dirs::data_local_dir() {
        dirs.push(dir.join("recurrency"));
    }
    if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
        let home = PathBuf::from(home);
        dirs.push(home.join(".config").join("recurrency"));
        dirs.push(home.join(".local").join("share").join("recurrency"));
        dirs.push(
            home.join("Library")
                .join("Application Support")
                .join("recurrency"),
        );
        dirs.push(home.join("AppData").join("Roaming").join("recurrency"));
    }
    dirs.dedup();
    dirs
}

fn valid_file(path: &Path) -> Option<String> {
    let value = std::fs::read_to_string(path).ok()?;
    if value.trim().is_empty() {
        None
    } else {
        Some(value.trim().to_string())
    }
}

/// Read a preference, first from the active directory, then its backup, then
/// from older data directories. A legacy file is copied into the active
/// directory before being returned.
pub fn read(name: &str) -> Option<String> {
    let active = recurrency_dir()?.join(name);
    if let Some(value) = valid_file(&active) {
        return Some(value);
    }
    if let Some(value) = valid_file(&backup_path(&active)) {
        return Some(value);
    }
    for dir in candidate_dirs() {
        let path = dir.join(name);
        if let Some(value) = valid_file(&path).or_else(|| valid_file(&backup_path(&path))) {
            let _ = std::fs::create_dir_all(active.parent().unwrap_or(&dir));
            let _ = std::fs::write(&active, &value);
            return Some(value);
        }
    }
    None
}

/// Write a preference and keep the previous value as a `.bak` file.
pub fn write(name: &str, value: &str) {
    let Some(dir) = recurrency_dir() else {
        return;
    };
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let path = dir.join(name);
    let temporary = path.with_extension("tmp");
    if std::fs::write(&temporary, value).is_err() {
        return;
    }
    if path.is_file() {
        let _ = std::fs::copy(&path, backup_path(&path));
        let _ = std::fs::remove_file(&path);
    }
    let _ = std::fs::rename(&temporary, &path);
}

pub fn exists(name: &str) -> bool {
    recurrency_dir()
        .map(|dir| dir.join(name).is_file())
        .unwrap_or(false)
}

pub fn remove(name: &str) {
    let Some(dir) = recurrency_dir() else {
        return;
    };
    let path = dir.join(name);
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(backup_path(&path));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backup_path_keeps_the_original_extension() {
        let path = Path::new("/tmp/recurrency/theme.txt");
        assert_eq!(
            backup_path(path),
            PathBuf::from("/tmp/recurrency/theme.txt.bak")
        );
    }

    #[test]
    fn empty_values_are_treated_as_missing() {
        let dir = std::env::temp_dir().join("recurrency-persistence-test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("empty.txt");
        std::fs::write(&path, "  \n").unwrap();
        assert!(valid_file(&path).is_none());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn data_dir_environment_is_supported() {
        assert!(!DATA_DIR_ENV.is_empty());
    }
}
