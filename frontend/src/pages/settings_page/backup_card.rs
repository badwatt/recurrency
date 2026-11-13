use dioxus::html::HasFileData;
use lucide_dioxus::{Download, FileText, ShieldCheck, Upload};
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

    let mut stage_text = move |text: String| {
        match serde_json::from_str::<Value>(&text) {
            Ok(value) => pending.set(Some(value)),
            Err(_) => TOAST.write().error("That file is not a Recurrency backup"),
        }
    };

    let open_android_backup = move |_| {
        spawn(async move {
            let mut eval = document::eval(r#"
                if (window.RecurrencyFilePicker) {
                    await new Promise((resolve) => {
                        window.__recurrencyBackupPicked = (data) => {
                            dioxus.send(data);
                            resolve();
                        };
                        window.RecurrencyFilePicker.pickJson();
                    });
                } else {
                    dioxus.send('');
                }
            "#);
            match eval.recv::<String>().await {
                Ok(text) if !text.is_empty() => stage_text(text),
                Ok(_) => {}
                Err(_) => TOAST.write().error("Could not open the file picker"),
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
        div { class: "rounded-xl bg-card/80 border border-border/40 overflow-hidden",
            div { class: "flex items-start gap-4 p-6 border-b border-border bg-gradient-to-br from-primary/5 to-transparent",
                span { class: "flex h-11 w-11 shrink-0 items-center justify-center rounded-xl bg-primary/10 text-primary",
                    ShieldCheck { size: 22 }
                }
                div {
                    h2 { class: "text-lg font-semibold", "Backup & restore" }
                    p { class: "text-sm text-muted-foreground", "Keep a portable copy of your data, or bring it back in one step." }
                }
                span { class: "ml-auto hidden sm:inline-flex items-center rounded-full border border-border/60 bg-background/70 px-2.5 py-1 text-[11px] font-medium text-muted-foreground",
                    "JSON"
                }
            }
            div { class: "p-6 grid gap-4 lg:grid-cols-2",
                // Export card
                div { class: "rounded-xl border border-border/60 bg-background/50 p-5 flex flex-col",
                    div { class: "flex items-center gap-3",
                        span { class: "flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-emerald-500/10 text-emerald-600 dark:text-emerald-400",
                            Download { size: 20 }
                        }
                        div {
                            h3 { class: "text-sm font-semibold", "Save a copy" }
                            p { class: "text-xs text-muted-foreground", "Download everything as one file." }
                        }
                    }
                    div { class: "mt-4 flex items-center gap-3 rounded-lg border border-dashed border-border/70 bg-muted/20 p-3",
                        span { class: "flex h-9 w-9 shrink-0 items-center justify-center rounded-md bg-background text-muted-foreground",
                            FileText { size: 18 }
                        }
                        div { class: "min-w-0",
                            p { class: "truncate text-sm font-medium", "recurrency-backup.json" }
                            p { class: "text-xs text-muted-foreground", "Accounts, subscriptions, transactions and debts" }
                        }
                    }
                    div { class: "mt-4 flex flex-1 flex-col items-center gap-3 text-center sm:flex-row sm:items-center sm:justify-between sm:text-left",
                        p { class: "text-xs text-muted-foreground", "Keep it private; it contains financial data." }
                        button {
                            r#type: "button",
                            class: "inline-flex h-9 shrink-0 items-center gap-2 rounded-md bg-primary px-3 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50",
                            disabled: exporting(),
                            onclick: export,
                            Download { size: 15 }
                            if exporting() { "Preparing…" } else { "Download backup" }
                        }
                    }
                }

                // Import card
                div { class: "rounded-xl border border-border/60 bg-background/50 p-5 flex flex-col",
                    div { class: "flex items-center gap-3",
                        span { class: "flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary",
                            Upload { size: 20 }
                        }
                        div {
                            h3 { class: "text-sm font-semibold", "Bring it back" }
                            p { class: "text-xs text-muted-foreground", "Restore from a previous backup file." }
                        }
                    }
                    div {
                        class: "mt-4 flex flex-1 flex-col items-center justify-center gap-2 rounded-lg border border-dashed border-border/70 bg-muted/10 p-5 text-center transition-colors hover:border-primary/50 hover:bg-primary/5",
                        ondragover: move |event: DragEvent| event.prevent_default(),
                        ondrop: move |event: DragEvent| {
                            event.prevent_default();
                            if let Some(file) = event.files().first().cloned() {
                                stage_file(file);
                            }
                        },
                        span { class: "flex h-9 w-9 items-center justify-center rounded-full bg-background text-muted-foreground",
                            Upload { size: 17 }
                        }
                        p { class: "text-sm font-medium", "Drop a backup here" }
                        p { class: "text-xs text-muted-foreground", "or choose a .json file" }
                        if cfg!(target_os = "android") {
                            button {
                                r#type: "button",
                                class: "mt-1 inline-flex h-9 cursor-pointer items-center justify-center rounded-md border border-border bg-background px-3 text-sm font-medium hover:bg-accent",
                                onclick: open_android_backup,
                                "Choose file"
                            }
                        } else {
                            label {
                                class: "relative mt-1 inline-flex h-9 cursor-pointer items-center justify-center rounded-md border border-border bg-background px-3 text-sm font-medium hover:bg-accent",
                                style: "cursor: pointer;",
                                "Choose file"
                                input {
                                    r#type: "file",
                                    accept: "application/json,.json",
                                    aria_label: "Choose backup file",
                                    class: "absolute inset-0 h-full w-full cursor-pointer opacity-0",
                                    style: "cursor: pointer;",
                                    onchange: move |event: Event<FormData>| {
                                        if let Some(file) = event.files().first().cloned() {
                                            stage_file(file);
                                        }
                                    },
                                }
                            }
                        }
                    }

                    if pending().is_some() {
                        div { class: "mt-4 rounded-lg border border-destructive/40 bg-destructive/10 p-4",
                            div { class: "flex items-center gap-2",
                                ShieldCheck { size: 16 }
                                p { class: "text-sm font-medium text-destructive", "Restore this backup?" }
                            }
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
                                    class: "inline-flex h-8 items-center rounded-md border border-border bg-background px-3 text-xs font-medium hover:bg-muted",
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
}
