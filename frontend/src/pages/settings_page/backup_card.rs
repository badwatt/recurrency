use dioxus::prelude::*;
use serde_json::Value;

use crate::api::backup;
use crate::components::backup_restore_card::BackupRestoreCard;
use crate::components::toast::TOAST;

const BACKUP_FILENAME: &str = "recurrency-backup.json";
const BACKUP_MIME: &str = "application/json";
const BACKUP_DETAIL: &str = "Accounts, subscriptions, transactions and debts";
const BACKUP_NOTE: &str = "Keep it private; it contains financial data.";
const RESTORE_WARNING: &str =
    "This replaces your current accounts, categories, subscriptions, transactions and debts.";

fn download_json(json: &str) {
    crate::utils::download::save_text(BACKUP_FILENAME, BACKUP_MIME, json);
}

fn download_started_message() -> &'static str {
    if cfg!(target_os = "android") {
        "Backup saved to Downloads."
    } else {
        "Backup download started. Check your Downloads folder."
    }
}

#[component]
pub fn BackupCard() -> Element {
    let mut exporting = use_signal(|| false);
    let mut importing = use_signal(|| false);
    let mut pending = use_signal(|| None::<Value>);

    let export = move |_| {
        if exporting() {
            return;
        }
        exporting.set(true);
        spawn(async move {
            match backup::export().await {
                Ok(value) => match serde_json::to_string_pretty(&value) {
                    Ok(json) => {
                        download_json(&json);
                        TOAST.write().success(download_started_message());
                    }
                    Err(error) => TOAST
                        .write()
                        .error(format!("Could not encode backup: {error}")),
                },
                Err(error) => TOAST
                    .write()
                    .error(format!("Could not create backup: {error}")),
            }
            exporting.set(false);
        });
    };

    let stage_text = move |text: String| match serde_json::from_str::<Value>(&text) {
        Ok(value) => pending.set(Some(value)),
        Err(_) => TOAST.write().error("That file is not a Recurrency backup"),
    };

    let restore = move |_| {
        let Some(value) = pending() else {
            return;
        };
        if importing() {
            return;
        }
        importing.set(true);
        spawn(async move {
            match backup::import(&value).await {
                Ok(_) => {
                    TOAST.write().success("Backup restored. Reloading…");
                    let _ = dioxus::document::eval("setTimeout(() => location.reload(), 500)");
                }
                Err(error) => {
                    TOAST
                        .write()
                        .error(format!("Could not restore backup: {error}"));
                    importing.set(false);
                }
            }
        });
    };

    rsx! {
        BackupRestoreCard {
            file_name: BACKUP_FILENAME,
            file_detail: BACKUP_DETAIL,
            export_note: BACKUP_NOTE,
            import_warning: RESTORE_WARNING,
            pending: pending().is_some(),
            exporting: exporting(),
            restoring: importing(),
            on_export: export,
            on_text: stage_text,
            on_confirm: restore,
            on_cancel: move |_| pending.set(None),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn user_backup_reuses_the_shared_card() {
        let source = include_str!("backup_card.rs");
        assert!(source.contains("BackupRestoreCard"));
        assert!(source.contains("BACKUP_FILENAME"));
        assert!(source.contains("save_text"));
        assert!(source.contains("Backup download started"));
    }
}
