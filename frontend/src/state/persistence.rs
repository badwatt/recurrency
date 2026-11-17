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
    #[cfg(test)]
    if let Some(dir) = TEST_DIR.lock().ok().and_then(|dir| dir.clone()) {
        return Some(dir);
    }
    crate::state::auth::get_app_data_dir().map(|dir| dir.join("recurrency"))
}

#[cfg(test)]
static TEST_DIR: std::sync::Mutex<Option<PathBuf>> = std::sync::Mutex::new(None);

fn backup_path(path: &Path) -> PathBuf {
    path.with_file_name(format!(
        "{}{BACKUP_SUFFIX}",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("config")
    ))
}

fn candidate_dirs() -> Vec<PathBuf> {
    #[cfg(test)]
    if let Some(dir) = TEST_DIR.lock().ok().and_then(|dir| dir.clone()) {
        return vec![dir];
    }
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

    static PERSISTENCE_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    struct PersistenceTest {
        _lock: std::sync::MutexGuard<'static, ()>,
        dir: PathBuf,
    }

    impl PersistenceTest {
        fn new() -> Self {
            let lock = PERSISTENCE_TEST_LOCK
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let dir = std::env::temp_dir().join(format!(
                "recurrency-persistence-{}",
                uuid::Uuid::new_v4()
            ));
            let _ = std::fs::remove_dir_all(&dir);
            *TEST_DIR
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(dir.clone());
            Self { _lock: lock, dir }
        }
    }

    impl Drop for PersistenceTest {
        fn drop(&mut self) {
            *TEST_DIR
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    fn dir() -> PathBuf {
        recurrency_dir().expect("test directory")
    }

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

    #[test]
    fn write_read_and_remove_keep_a_backup() {
        let _test = PersistenceTest::new();
        write("theme.txt", "dark");
        assert_eq!(read("theme.txt").as_deref(), Some("dark"));
        assert!(exists("theme.txt"));

        write("theme.txt", "light");
        let path = dir().join("theme.txt");
        assert_eq!(read("theme.txt").as_deref(), Some("light"));
        assert_eq!(
            std::fs::read_to_string(path.with_extension("txt.bak")).unwrap(),
            "dark"
        );

        remove("theme.txt");
        assert!(!exists("theme.txt"));
        assert!(read("theme.txt").is_none());
        assert!(!path.with_extension("txt.bak").exists());
    }

    #[test]
    fn read_prefers_the_active_value_and_falls_back_to_the_backup() {
        let _test = PersistenceTest::new();
        write("app_mode.txt", "offline");
        write("app_mode.txt", "server\nhttps://example.test");
        assert_eq!(
            read("app_mode.txt").as_deref(),
            Some("server\nhttps://example.test")
        );

        std::fs::remove_file(dir().join("app_mode.txt")).unwrap();
        assert_eq!(read("app_mode.txt").as_deref(), Some("offline"));
    }

    #[test]
    fn blank_active_values_use_the_backup() {
        let _test = PersistenceTest::new();
        write("server_url.txt", "https://first.test");
        write("server_url.txt", "https://second.test");
        std::fs::write(dir().join("server_url.txt"), "   \n").unwrap();
        assert_eq!(
            read("server_url.txt").as_deref(),
            Some("https://first.test")
        );
    }

    #[test]
    fn missing_and_blank_values_are_not_returned() {
        let _test = PersistenceTest::new();
        assert!(read("missing.txt").is_none());
        std::fs::create_dir_all(dir()).unwrap();
        std::fs::write(dir().join("blank.txt"), " \n ").unwrap();
        assert!(read("blank.txt").is_none());
    }

    #[test]
    fn remove_missing_preferences_is_safe() {
        let _test = PersistenceTest::new();
        remove("never-written.txt");
        assert!(!exists("never-written.txt"));
    }

    #[test]
    fn writes_create_the_data_directory() {
        let _test = PersistenceTest::new();
        assert!(!dir().exists());
        write("theme.txt", "mocha");
        assert!(dir().join("theme.txt").is_file());
    }
    #[test]
    fn backup_path_falls_back_to_config_for_a_directory() {
        assert_eq!(
            backup_path(Path::new("/tmp/recurrency")),
            PathBuf::from("/tmp/recurrency.bak")
        );
    }

    #[test]
    fn write_replaces_an_existing_backup_and_remove_is_idempotent() {
        let _test = PersistenceTest::new();
        write("view.txt", "grid");
        write("view.txt", "list");
        write("view.txt", "table");
        assert_eq!(read("view.txt").as_deref(), Some("table"));
        assert_eq!(
            std::fs::read_to_string(dir().join("view.txt.bak")).unwrap(),
            "list"
        );
        remove("missing.txt");
        assert!(!exists("missing.txt"));
    }

}
