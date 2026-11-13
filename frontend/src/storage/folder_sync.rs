//! Android SAF folder sync for local mode.

use crate::storage::local;
use dioxus::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};

static SYNCING: AtomicBool = AtomicBool::new(false);

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

/// Read every device file in the selected folder, merge it into local data and
/// write this device's merged file back. Safe to call repeatedly.
pub async fn sync_once() -> Result<(), String> {
    if SYNCING.swap(true, Ordering::SeqCst) {
        return Ok(());
    }
    let result = async {
        let (linked, _) = status().await;
        if !linked {
            return Ok(());
        }
        let mut eval = document::eval(
            r#"
            const bridge = window.RecurrencySyncFolder;
            dioxus.send(bridge && bridge.readAll ? bridge.readAll() : '[]');
            "#,
        );
        let remote_raw = eval.recv::<String>().await.unwrap_or_else(|_| "[]".to_string());
        let remote: Vec<String> = serde_json::from_str(&remote_raw).unwrap_or_default();
        let merged = local::sync_merge(&remote).await.map_err(|error| error.to_string())?;
        let mut eval = document::eval(
            r#"
            const content = await dioxus.recv();
            const bridge = window.RecurrencySyncFolder;
            dioxus.send(String(!!(bridge && bridge.writeFile && bridge.writeFile(content))));
            "#,
        );
        let _ = eval.send(merged);
        let written = eval.recv::<String>().await.unwrap_or_else(|_| "false".to_string());
        if written != "true" {
            return Err("Could not write the sync file".to_string());
        }
        Ok(())
    }
    .await;
    SYNCING.store(false, Ordering::SeqCst);
    result
}
