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
                    button {
                        class: "border px-3 py-1.5 rounded-md text-sm hover:bg-muted disabled:opacity-50",
                        disabled: deleting(),
                        onclick: move |_| {
                            if !deleting() {
                                confirm_id.set(None);
                            }
                        },
                        "Cancel"
                    }
                    button {
                        class: "bg-red-600 text-white px-3 py-1.5 rounded-md text-sm hover:bg-red-700 disabled:opacity-50",
                        disabled: deleting(),
                        onclick: move |_| {
                            if deleting() {
                                return;
                            }
                            deleting.set(true);
                            let id = confirm_id().unwrap();
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
