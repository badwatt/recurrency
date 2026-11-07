use crate::models::subscription::CalendarItem;
use crate::state::subscriptions::SUBSCRIPTIONS;
use crate::storage::Storage;
use dioxus::prelude::*;
use uuid::Uuid;

/// Confirmation modal shown before deleting a subscription. Mounted when
/// `confirm_id` is `Some`.
#[component]
pub fn DeleteSubscriptionDialog(
    confirm_id: Signal<Option<Uuid>>,
    calendar_items: Signal<Vec<CalendarItem>>,
    current_year: Signal<i32>,
    current_month: Signal<u32>,
) -> Element {
    let mut deleting = use_signal(|| false);
    let handle_delete_confirm = move |_| {
        if deleting() {
            return;
        }
        if let Some(id) = confirm_id() {
            deleting.set(true);
            spawn(async move {
                let _ = Storage::delete_subscription(&id).await;
                match Storage::list_subscriptions().await {
                    Ok(data) => SUBSCRIPTIONS.write().set_data(data),
                    Err(e) => SUBSCRIPTIONS.write().set_error(e.to_string()),
                }
                // Also refresh calendar
                let y = current_year();
                let m = current_month();
                if let Ok(items) = Storage::calendar_month(y, m).await {
                    calendar_items.set(items)
                }
                // Close after refreshing: closing unmounts this component and
                // cancels its in-flight tasks.
                confirm_id.set(None);
            });
        }
    };

    rsx! {
        div { class: "fixed inset-0 z-[70] flex items-center justify-center bg-black/50",
            div { class: "bg-background rounded-none sm:rounded-lg p-4 sm:p-6 w-full sm:w-80 space-y-4 shadow-lg h-dvh sm:h-auto flex flex-col",
                h3 { class: "font-semibold", "Delete subscription?" }
                p { class: "text-sm text-muted-foreground",
                    "This action cannot be undone."
                }
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
                        onclick: handle_delete_confirm,
                        if deleting() { "Deleting..." } else { "Delete" }
                    }
                }
            }
        }
    }
}
