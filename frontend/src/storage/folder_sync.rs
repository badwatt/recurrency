//! Cross-platform folder sync for local native mode.
//!
//! Android uses the SAF bridge; desktop uses a native folder picker and direct
//! filesystem access. Web clients never expose local sync.

use crate::storage::local;
use std::sync::atomic::{AtomicBool, Ordering};

static SYNCING: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "android")]
mod platform {
    use dioxus::prelude::*;

    pub async fn status() -> (bool, String) {
        let mut eval = document::eval(
            r#"
            const bridge = window.RecurrencySyncFolder;
            dioxus.send(JSON.stringify({
                linked: !!(bridge && bridge.hasFolder && bridge.hasFolder()),
                name: bridge && bridge.folderName ? bridge.folderName() : "",
            }));
            "#,
        );
        match eval.recv::<String>().await {
            Ok(raw) => serde_json::from_str::<serde_json::Value>(&raw)
                .ok()
                .map(|value| {
                    (
                        value["linked"].as_bool().unwrap_or(false),
                        value["name"].as_str().unwrap_or_default().to_string(),
                    )
                })
                .unwrap_or((false, String::new())),
            Err(_) => (false, String::new()),
        }
    }

    pub async fn choose_folder() -> Result<(), String> {
        let mut eval = document::eval(
            r#"
            if (!window.RecurrencySyncFolder) {
                dioxus.send('');
                return;
            }
            await new Promise((resolve) => {
                window.__recurrencySyncFolderChosen = (value) => {
                    dioxus.send(value);
                    resolve();
                };
                window.RecurrencySyncFolder.chooseFolder();
            });
            "#,
        );
        match eval.recv::<String>().await {
            Ok(value) if value == "ok" => Ok(()),
            Ok(_) => Err("No folder selected".to_string()),
            Err(_) => Err("Could not open the folder picker".to_string()),
        }
    }

    pub async fn disconnect() {
        let _ = document::eval(
            "window.RecurrencySyncFolder && window.RecurrencySyncFolder.disconnect && window.RecurrencySyncFolder.disconnect();",
        );
    }

    pub async fn read_all() -> Result<Vec<String>, String> {
        let mut eval = document::eval(
            r#"
            const bridge = window.RecurrencySyncFolder;
            dioxus.send(bridge && bridge.readAll ? bridge.readAll() : '[]');
            "#,
        );
        let raw = eval.recv::<String>().await.unwrap_or_else(|_| "[]".to_string());
        Ok(serde_json::from_str(&raw).unwrap_or_default())
    }

    pub async fn write_file(content: &str) -> Result<(), String> {
        let mut eval = document::eval(
            r#"
            const content = await dioxus.recv();
            const bridge = window.RecurrencySyncFolder;
            dioxus.send(String(!!(bridge && bridge.writeFile && bridge.writeFile(content))));
            "#,
        );
        let _ = eval.send(content.to_string());
        let written = eval.recv::<String>().await.unwrap_or_else(|_| "false".to_string());
        if written == "true" {
            Ok(())
        } else {
            Err("Could not write the sync file".to_string())
        }
    }
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android"), feature = "desktop"))]
mod platform {
    use std::path::{Path, PathBuf};

    const SYNC_FOLDER_FILE: &str = "sync_folder.txt";
    const DEVICE_FILE: &str = "sync_device.txt";

    fn app_dir() -> Option<PathBuf> {
        crate::state::auth::get_app_data_dir().map(|dir| dir.join("recurrency"))
    }

    fn folder_file() -> Option<PathBuf> {
        app_dir().map(|dir| dir.join(SYNC_FOLDER_FILE))
    }

    fn selected_folder() -> Option<PathBuf> {
        let raw = std::fs::read_to_string(folder_file()?).ok()?;
        let path = PathBuf::from(raw.trim());
        path.is_dir().then_some(path)
    }

    fn device_id() -> Result<String, String> {
        let dir = app_dir().ok_or_else(|| "No data directory".to_string())?;
        std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
        let file = dir.join(DEVICE_FILE);
        if let Ok(value) = std::fs::read_to_string(&file)
            && !value.trim().is_empty()
        {
            return Ok(value.trim().to_string());
        }
        let id = uuid::Uuid::new_v4().to_string();
        std::fs::write(&file, &id).map_err(|error| error.to_string())?;
        Ok(id)
    }

    pub async fn status() -> (bool, String) {
        match selected_folder() {
            Some(path) => (
                true,
                path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("Sync folder")
                    .to_string(),
            ),
            None => (false, String::new()),
        }
    }

    pub async fn choose_folder() -> Result<(), String> {
        let folder = rfd::AsyncFileDialog::new()
            .set_title("Choose a Recurrency sync folder")
            .pick_folder()
            .await
            .ok_or_else(|| "No folder selected".to_string())?;
        let dir = app_dir().ok_or_else(|| "No data directory".to_string())?;
        std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
        std::fs::write(dir.join(SYNC_FOLDER_FILE), folder.path().display().to_string())
            .map_err(|error| error.to_string())
    }

    pub async fn disconnect() {
        if let Some(file) = folder_file() {
            let _ = std::fs::remove_file(file);
        }
    }

    fn is_sync_file(path: &Path) -> bool {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("recurrency-sync-") && name.ends_with(".json"))
    }

    pub async fn read_all() -> Result<Vec<String>, String> {
        let folder = selected_folder().ok_or_else(|| "No sync folder".to_string())?;
        let mut files = Vec::new();
        for entry in std::fs::read_dir(&folder).map_err(|error| error.to_string())? {
            let path = entry.map_err(|error| error.to_string())?.path();
            if path.is_file() && is_sync_file(&path) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    files.push(content);
                }
            }
        }
        Ok(files)
    }

    pub async fn write_file(content: &str) -> Result<(), String> {
        let folder = selected_folder().ok_or_else(|| "No sync folder".to_string())?;
        let device = device_id()?;
        let path = folder.join(format!("recurrency-sync-{device}.json"));
        std::fs::write(path, content).map_err(|error| error.to_string())
    }
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android"), not(feature = "desktop")))]
mod platform {
    pub async fn status() -> (bool, String) {
        (false, String::new())
    }
    pub async fn choose_folder() -> Result<(), String> {
        Err("Folder sync is available in native desktop builds".to_string())
    }
    pub async fn disconnect() {}
    pub async fn read_all() -> Result<Vec<String>, String> {
        Ok(Vec::new())
    }
    pub async fn write_file(_content: &str) -> Result<(), String> {
        Err("Folder sync is available in native desktop builds".to_string())
    }
}

#[cfg(target_arch = "wasm32")]
mod platform {
    pub async fn status() -> (bool, String) {
        (false, String::new())
    }
    pub async fn choose_folder() -> Result<(), String> {
        Err("Folder sync is available in native builds".to_string())
    }
    pub async fn disconnect() {}
    pub async fn read_all() -> Result<Vec<String>, String> {
        Ok(Vec::new())
    }
    pub async fn write_file(_content: &str) -> Result<(), String> {
        Err("Folder sync is available in native builds".to_string())
    }
}

/// True only when the app is running as a native local-mode client.
pub fn is_available() -> bool {
    cfg!(not(target_arch = "wasm32")) && crate::state::app_mode::APP_MODE().mode.is_offline()
}

pub async fn status() -> (bool, String) {
    platform::status().await
}

pub async fn choose_folder() -> Result<(), String> {
    platform::choose_folder().await
}

pub async fn disconnect() {
    platform::disconnect().await
}

/// Read every device file, merge it into local data and write this device's
/// merged file back. Safe to call repeatedly.
pub async fn sync_once() -> Result<(), String> {
    if SYNCING.swap(true, Ordering::SeqCst) {
        return Ok(());
    }
    let result = async {
        let (linked, _) = status().await;
        if !linked {
            return Ok(());
        }
        let remote = platform::read_all().await?;
        let merged = local::sync_merge(&remote)
            .await
            .map_err(|error| error.to_string())?;
        platform::write_file(&merged).await?;
        Ok(())
    }
    .await;
    SYNCING.store(false, Ordering::SeqCst);
    result
}

#[cfg(test)]
mod tests {
    #[test]
    fn native_builds_expose_folder_sync() {
        let source = include_str!("folder_sync.rs");
        assert!(source.contains("AsyncFileDialog"));
        assert!(source.contains("cfg!(not(target_arch = \"wasm32\"))"));
    }
}
