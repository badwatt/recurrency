//! Local folder sync for the native app. The folder is chosen once and every
//! manual sync writes a validated server backup into it, so a cloud client
//! (Drive, OneDrive, Dropbox) can upload the file.

use std::path::{Path, PathBuf};

use nsh_core::storage::Storage;

const FOLDER_KEY: &str = "recurrency.sync-folder";
const LAST_SYNC_KEY: &str = "recurrency.sync-last";
pub const BACKUP_FILE: &str = "recurrency-backup.json";

pub fn folder() -> Option<PathBuf> {
    Storage::get_item(FOLDER_KEY)
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
}

pub fn set_folder(path: &Path) {
    let _ = Storage::set_item(FOLDER_KEY, path.to_string_lossy().as_ref());
}

pub fn disconnect() {
    let _ = Storage::remove_item(FOLDER_KEY);
    let _ = Storage::remove_item(LAST_SYNC_KEY);
}

pub fn last_sync() -> String {
    Storage::get_item(LAST_SYNC_KEY).unwrap_or_default()
}

pub fn display_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string())
}

pub async fn sync(server: String, token: String) -> Result<PathBuf, String> {
    let folder = folder().ok_or_else(|| "Choose a sync folder first.".to_string())?;
    let backup = crate::api::export_backup(server, token).await?;
    std::fs::create_dir_all(&folder)
        .map_err(|error| format!("Could not open the sync folder: {error}"))?;
    let target = folder.join(BACKUP_FILE);
    std::fs::write(&target, backup)
        .map_err(|error| format!("Could not write the backup: {error}"))?;
    let _ = Storage::set_item(
        LAST_SYNC_KEY,
        chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
    );
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_folder_name_is_shown_without_its_parents() {
        assert_eq!(display_name(Path::new("/home/me/Recurrency")), "Recurrency");
    }

    #[test]
    fn the_backup_file_name_is_stable() {
        assert_eq!(BACKUP_FILE, "recurrency-backup.json");
        assert!(!BACKUP_FILE.contains('/'));
    }
}
