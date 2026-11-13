use crate::components::toast::TOAST;
use crate::state::app_mode::APP_MODE;
use crate::storage::local;
use dioxus::prelude::*;

async fn sync_status() -> (bool, String) {
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

#[component]
pub fn SyncFolderCard() -> Element {
    let local_mode = cfg!(target_os = "android") && APP_MODE().mode.is_offline();
    let mut linked = use_signal(|| false);
    let mut folder_name = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(String::new);
    let mut last_sync = use_signal(String::new);

    use_future(move || async move {
        if !local_mode {
            return;
        }
        let (has_folder, name) = sync_status().await;
        linked.set(has_folder);
        folder_name.set(name);
    });

    let choose = move |_| {
        if busy() {
            return;
        }
        busy.set(true);
        error.set(String::new());
        spawn(async move {
            let mut eval = document::eval(r#"
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
            "#);
            match eval.recv::<String>().await {
                Ok(value) if value == "ok" => {
                    let (has_folder, name) = sync_status().await;
                    linked.set(has_folder);
                    folder_name.set(name);
                    TOAST.write().success("Sync folder connected");
                }
                Ok(_) => error.set("No folder selected".to_string()),
                Err(_) => error.set("Could not open the folder picker".to_string()),
            }
            busy.set(false);
        });
    };

    let sync_now = move |_| {
        if busy() || !linked() {
            return;
        }
        busy.set(true);
        error.set(String::new());
        spawn(async move {
            let result = async {
                let mut eval = document::eval(r#"
                    const bridge = window.RecurrencySyncFolder;
                    dioxus.send(bridge && bridge.readAll ? bridge.readAll() : '[]');
                "#);
                let remote_raw = eval.recv::<String>().await.unwrap_or_else(|_| "[]".to_string());
                let remote: Vec<String> = serde_json::from_str(&remote_raw).unwrap_or_default();
                let merged = local::sync_merge(&remote).await?;
                let mut eval = document::eval(r#"
                    const content = await dioxus.recv();
                    const bridge = window.RecurrencySyncFolder;
                    dioxus.send(String(!!(bridge && bridge.writeFile && bridge.writeFile(content))));
                "#);
                let _ = eval.send(merged);
                let written = eval.recv::<String>().await.unwrap_or_else(|_| "false".to_string());
                if written != "true" {
                    return Err(crate::api::client::ApiError::Network(
                        "Could not write the sync file".to_string(),
                    ));
                }
                Ok::<(), crate::api::client::ApiError>(())
            }
            .await;
            match result {
                Ok(()) => {
                    last_sync.set(chrono::Local::now().format("%H:%M").to_string());
                    TOAST.write().success("Sync complete");
                }
                Err(err) => error.set(err.to_string()),
            }
            busy.set(false);
        });
    };

    let disconnect = move |_| {
        spawn(async move {
            let _ = document::eval(
                "window.RecurrencySyncFolder && window.RecurrencySyncFolder.disconnect && window.RecurrencySyncFolder.disconnect();",
            );
            linked.set(false);
            folder_name.set(String::new());
            TOAST.write().success("Sync folder disconnected");
        });
    };

    if !local_mode {
        return rsx! {};
    }

    rsx! {
        div { class: "rounded-xl bg-card/80 border border-border/40",
            div { class: "p-6 border-b border-border",
                h2 { class: "text-lg font-semibold", "Cloud folder sync" }
                p { class: "text-sm text-muted-foreground",
                    "Keep local data in sync through a folder in Google Drive, OneDrive, Dropbox or another document provider."
                }
            }
            div { class: "p-6 space-y-4",
                div { class: "flex items-center gap-3 rounded-lg border border-border/60 bg-background/50 p-4",
                    div { class: "flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10 text-primary", "☁" }
                    div { class: "min-w-0 flex-1",
                        p { class: "text-sm font-medium", if linked() { "Folder connected" } else { "No sync folder" } }
                        p { class: "truncate text-xs text-muted-foreground",
                            {if linked() { folder_name() } else { "Choose a folder in your cloud provider.".to_string() }}
                        }
                    }
                    if linked() {
                        button {
                            r#type: "button",
                            class: "text-xs text-muted-foreground hover:text-foreground",
                            onclick: disconnect,
                            "Disconnect"
                        }
                    }
                }
                if !error().is_empty() {
                    p { class: "text-sm text-destructive", {error()} }
                }
                div { class: "flex flex-wrap items-center gap-2",
                    button {
                        r#type: "button",
                        class: "inline-flex h-9 items-center rounded-md border border-border px-3 text-sm font-medium hover:bg-accent disabled:opacity-50",
                        disabled: busy(),
                        onclick: choose,
                        if linked() { "Change folder" } else { "Choose folder" }
                    }
                    button {
                        r#type: "button",
                        class: "inline-flex h-9 items-center rounded-md bg-primary px-3 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50",
                        disabled: busy() || !linked(),
                        onclick: sync_now,
                        if busy() { "Syncing…" } else { "Sync now" }
                    }
                    if !last_sync().is_empty() {
                        span { class: "text-xs text-muted-foreground", "Last sync {last_sync()}" }
                    }
                }
            }
        }
    }
}
