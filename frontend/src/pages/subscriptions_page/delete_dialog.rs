use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::dialog::Dialog;
use crate::state::subscriptions::SUBSCRIPTIONS;
use crate::storage::Storage;
use dioxus::prelude::*;
use uuid::Uuid;

/// Delete-subscription confirmation modal. Mounted when `confirm_id` is `Some`.
#[component]
pub fn DeleteSubscriptionDialog(confirm_id: Signal<Option<Uuid>>) -> Element {
    let mut deleting = use_signal(|| false);
    rsx! {
        Dialog {
            open: true,
            on_close: move |_| {
                if !deleting() {
                    confirm_id.set(None);
                }
            },
            title: "Delete subscription?".to_string(),
            footer: rsx! {
                div { class: "flex flex-col-reverse sm:flex-row sm:justify-end gap-2",
                    Button {
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Sm,
                        disabled: deleting(),
                        onclick: move |_| {
                            if !deleting() {
                                confirm_id.set(None);
                            }
                        },
                        "Cancel"
                    }
                    Button {
                        variant: ButtonVariant::Destructive,
                        size: ButtonSize::Sm,
                        disabled: deleting(),
                        onclick: move |_| {
                            if deleting() {
                                return;
                            }
                            deleting.set(true);
                            let Some(id) = confirm_id() else { return };

                            spawn(async move {
                                let _ = Storage::delete_subscription(&id).await;
                                match Storage::list_subscriptions().await {
                                    Ok(data) => SUBSCRIPTIONS.write().set_data(data),
                                    Err(e) => SUBSCRIPTIONS.write().set_error(e.to_string()),
                                }
                                // Close after refreshing: closing unmounts this
                                // component and cancels its in-flight tasks.
                                confirm_id.set(None);
                            });
                        },
                        if deleting() { "Deleting..." } else { "Delete" }
                    }
                }
            },
            p { class: "text-sm text-muted-foreground", "This action cannot be undone." }
        }
    }
}

/// Bulk-delete confirmation. The dialog stays mounted while `ids` is non-empty.
#[component]
pub fn BulkDeleteSubscriptionsDialog(ids: Signal<Vec<Uuid>>, on_done: EventHandler<()>) -> Element {
    let mut deleting = use_signal(|| false);
    let count = ids().len();
    rsx! {
        Dialog {
            open: true,
            on_close: move |_| {
                if !deleting() {
                    ids.set(Vec::new());
                }
            },
            title: format!("Delete {count} subscription{}?", if count == 1 { "" } else { "s" }),
            footer: rsx! {
                div { class: "flex flex-col-reverse sm:flex-row sm:justify-end gap-2",
                    Button {
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Sm,
                        disabled: deleting(),
                        onclick: move |_| {
                            if !deleting() {
                                ids.set(Vec::new());
                            }
                        },
                        "Cancel"
                    }
                    Button {
                        variant: ButtonVariant::Destructive,
                        size: ButtonSize::Sm,
                        disabled: deleting(),
                        onclick: move |_| {
                            if deleting() {
                                return;
                            }
                            deleting.set(true);
                            let to_delete = ids();
                            spawn(async move {
                                for id in to_delete {
                                    let _ = Storage::delete_subscription(&id).await;
                                }
                                match Storage::list_subscriptions().await {
                                    Ok(data) => SUBSCRIPTIONS.write().set_data(data),
                                    Err(e) => SUBSCRIPTIONS.write().set_error(e.to_string()),
                                }
                                ids.set(Vec::new());
                                on_done.call(());
                            });
                        },
                        if deleting() { "Deleting..." } else { "Delete" }
                    }
                }
            },
            p { class: "text-sm text-muted-foreground", "This action cannot be undone." }
        }
    }
}
