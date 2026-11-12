use dioxus::html::HasFileData;
use crate::api::backup;
use crate::components::toast::TOAST;
use dioxus::prelude::*;
use serde_json::Value;

fn download_json(json: &str) {
    crate::utils::download::save_text("recurrency-backup.json", "application/json", json);
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
                    Ok(json) => download_json(&json),
                    Err(error) => TOAST.write().error(format!("Could not encode backup: {error}")),
                },
                Err(error) => TOAST.write().error(format!("Could not create backup: {error}")),
            }
            exporting.set(false);
        });
    };

    let stage_file = move |file: dioxus::html::FileData| {
        spawn(async move {
            match file.read_string().await {
                Ok(text) => match serde_json::from_str::<Value>(&text) {
                    Ok(value) => pending.set(Some(value)),
                    Err(_) => TOAST.write().error("That file is not a Recurrency backup"),
                },
                Err(error) => TOAST.write().error(format!("Could not read backup: {error}")),
            }
        });
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
                    TOAST.write().error(format!("Could not restore backup: {error}"));
                    importing.set(false);
                }
            }
        });
    };

    rsx! {
        div { class: "rounded-xl bg-card/80 border border-border/40",
            div { class: "p-6 border-b border-border",
                h2 { class: "text-lg font-semibold", "Backup & restore" }
                p { class: "text-sm text-muted-foreground", "Download your accounts, subscriptions, transactions, debts and payment history as JSON." }
            }
            div { class: "p-6 space-y-4",
                div { class: "flex flex-col items-center gap-3 rounded-lg border border-border/50 p-4 text-center",
                    div {
                        p { class: "text-sm font-medium", "Download a backup" }
                        p { class: "text-xs text-muted-foreground", "Keep it somewhere safe; it contains your financial data." }
                    }
                    button {
                        r#type: "button",
                        class: "inline-flex h-9 items-center rounded-md border border-border px-3 text-sm font-medium hover:bg-accent disabled:opacity-50",
                        disabled: exporting(),
                        onclick: export,
                        if exporting() { "Preparing…" } else { "Download backup" }
                    }
                }

                div {
                    class: "rounded-lg border border-dashed border-border/70 p-6 text-center transition-colors hover:border-primary/50 hover:bg-primary/5",
                    ondragover: move |event: DragEvent| event.prevent_default(),
                    ondrop: move |event: DragEvent| {
                        event.prevent_default();
                        if let Some(file) = event.files().first().cloned() {
                            stage_file(file);
                        }
                    },
                    p { class: "text-sm font-medium", "Drop a backup here" }
                    p { class: "mt-1 text-xs text-muted-foreground", "or choose a .json file" }
                    label {
                        class: "relative mt-3 inline-flex h-9 cursor-pointer items-center justify-center rounded-md border border-border px-3 text-sm font-medium hover:bg-accent",
                        "Choose file"
                        input {
                            r#type: "file",
                            accept: "application/json,.json",
                            aria_label: "Choose backup file",
                            class: "absolute inset-0 h-full w-full cursor-pointer opacity-0",
                            onchange: move |event: Event<FormData>| {
                                if let Some(file) = event.files().first().cloned() {
                                    stage_file(file);
                                }
                            },
                        }
                    }
                }

                if pending().is_some() {
                    div { class: "rounded-lg border border-destructive/40 bg-destructive/10 p-4",
                        p { class: "text-sm font-medium text-destructive", "Restore this backup?" }
                        p { class: "mt-1 text-xs text-muted-foreground", "This replaces your current accounts, categories, subscriptions, transactions and debts." }
                        div { class: "mt-3 flex flex-wrap gap-2",
                            button {
                                r#type: "button",
                                class: "inline-flex h-8 items-center rounded-md bg-destructive px-3 text-xs font-medium text-white hover:bg-destructive/90 disabled:opacity-50",
                                disabled: importing(),
                                onclick: restore,
                                if importing() { "Restoring…" } else { "Restore backup" }
                            }
                            button {
                                r#type: "button",
                                class: "inline-flex h-8 items-center rounded-md border border-border px-3 text-xs font-medium hover:bg-muted",
                                onclick: move |_| pending.set(None),
                                "Cancel"
                            }
                        }
                    }
                }
            }
        }
    }
}
