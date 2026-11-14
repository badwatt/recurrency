use crate::components::toast::TOAST;
use crate::storage::folder_sync;
use dioxus::prelude::*;

#[component]
pub fn SyncFolderCard() -> Element {
    let local_mode = crate::storage::folder_sync::is_available();
    let mut linked = use_signal(|| false);
    let mut folder_name = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(String::new);
    let mut last_sync = use_signal(String::new);

    use_future(move || async move {
        if !local_mode {
            return;
        }
        let (has_folder, name) = folder_sync::status().await;
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
            match folder_sync::choose_folder().await {
                Ok(()) => {
                    let (has_folder, name) = folder_sync::status().await;
                    linked.set(has_folder);
                    folder_name.set(name);
                    TOAST.write().success("Sync folder connected");
                }
                Err(message) => error.set(message),
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
            let result = folder_sync::sync_once().await;
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
            folder_sync::disconnect().await;
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
