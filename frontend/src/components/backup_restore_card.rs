use crate::components::button::{Button, ButtonSize, ButtonVariant};
use dioxus::html::HasFileData;
use dioxus::prelude::*;
use lucide_dioxus::{Download, FileText, ShieldCheck, Upload};

use crate::components::toast::TOAST;

/// Shared visual language for full backups and per-user backups.
///
/// The two screens differ only in the backend operation, file name and the
/// warning copy; the layout stays identical.
#[component]
pub fn BackupRestoreCard(
    file_name: String,
    file_detail: String,
    export_note: String,
    import_warning: String,
    pending: bool,
    exporting: bool,
    restoring: bool,
    on_export: EventHandler<()>,
    on_text: EventHandler<String>,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let stage = move |file: dioxus::html::FileData| {
        spawn(async move {
            match file.read_string().await {
                Ok(text) => on_text.call(text),
                Err(error) => TOAST
                    .write()
                    .error(format!("Could not read backup: {error}")),
            }
        });
    };

    let open_android_backup = move |_| {
        spawn(async move {
            let mut eval = document::eval(
                r#"
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
                "#,
            );
            match eval.recv::<String>().await {
                Ok(text) if !text.is_empty() => on_text.call(text),
                Ok(_) => {}
                Err(_) => TOAST.write().error("Could not open the file picker"),
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
                    p { class: "text-sm text-muted-foreground",
                        "Keep a portable copy of your data, or bring it back in one step."
                    }
                }
                span { class: "ml-auto hidden sm:inline-flex items-center rounded-full border border-border/60 bg-background/70 px-2.5 py-1 text-[11px] font-medium text-muted-foreground",
                    "JSON"
                }
            }
            div { class: "p-6 grid gap-4 lg:grid-cols-2",
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
                            p { class: "truncate text-sm font-medium", {file_name} }
                            p { class: "text-xs text-muted-foreground", {file_detail} }
                        }
                    }
                    div { class: "mt-4 flex flex-1 flex-col items-center gap-3 text-center sm:flex-row sm:items-center sm:justify-between sm:text-left",
                        p { class: "text-xs text-muted-foreground", {export_note} }
                        Button {
                            size: ButtonSize::Sm,
                            class: "shrink-0",
                            disabled: exporting,
                            onclick: move |_| on_export.call(()),
                            Download { size: 15 }
                            if exporting { "Preparing…" } else { "Download backup" }
                        }
                    }
                }

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
                                stage(file);
                            }
                        },
                        span { class: "flex h-9 w-9 items-center justify-center rounded-full bg-background text-muted-foreground",
                            Upload { size: 17 }
                        }
                        p { class: "text-sm font-medium", "Drop a backup here" }
                        p { class: "text-xs text-muted-foreground", "or choose a .json file" }
                        if cfg!(target_os = "android") {
                            Button {
                                variant: ButtonVariant::Outline,
                                class: "mt-1",
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
                                    id: "backup-file",
                                    name: "backup",
                                    autocomplete: "off",
                                    accept: "application/json,.json",
                                    aria_label: "Choose backup file",
                                    class: "absolute inset-0 h-full w-full cursor-pointer opacity-0",
                                    style: "cursor: pointer; pointer-events: none;",
                                    onchange: move |event: Event<FormData>| {
                                        if let Some(file) = event.files().first().cloned() {
                                            stage(file);
                                        }
                                    },
                                }
                            }
                        }
                    }

                    if pending {
                        div { class: "mt-4 rounded-lg border border-destructive/40 bg-destructive/10 p-4",
                            div { class: "flex items-center gap-2",
                                ShieldCheck { size: 16 }
                                p { class: "text-sm font-medium text-destructive", "Restore this backup?" }
                            }
                            p { class: "mt-1 text-xs text-muted-foreground", {import_warning} }
                            div { class: "mt-3 flex flex-wrap gap-2",
                                Button {
                                    variant: ButtonVariant::Destructive,
                                    size: ButtonSize::Sm,
                                    disabled: restoring,
                                    onclick: move |_| on_confirm.call(()),
                                    if restoring { "Restoring…" } else { "Restore backup" }
                                }
                                Button {
                                    variant: ButtonVariant::Outline,
                                    size: ButtonSize::Sm,
                                    onclick: move |_| on_cancel.call(()),
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

#[cfg(test)]
mod tests {
    #[test]
    fn shared_backup_card_is_the_single_visual_source() {
        let source = include_str!("backup_restore_card.rs");
        assert!(source.contains("Backup & restore"));
        assert!(source.contains("Save a copy"));
        assert!(source.contains("Bring it back"));
        assert!(source.contains("Drop a backup here"));
        assert!(source.contains("Download backup"));
        assert!(source.contains("Restore backup"));
        assert!(source.contains("RecurrencyFilePicker"));
        assert!(source.contains("__recurrencyBackupPicked"));
        assert!(source.contains("sm:flex-row"));
        assert!(source.contains("sm:justify-between"));
    }
}
