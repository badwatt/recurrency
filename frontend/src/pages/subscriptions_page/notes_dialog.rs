use crate::components::dialog::Dialog;
use crate::models::subscription::UpdateSubscriptionRequest;
use crate::state::subscriptions::SUBSCRIPTIONS;
use crate::storage::Storage;
use dioxus::prelude::*;
use uuid::Uuid;

/// Notes editor for a subscription. Keeps the existing page-owned signals so
/// the parent can open it directly from a card's notes button.
#[component]
pub fn NotesSubscriptionDialog(
    open: Signal<bool>,
    sub_id: Signal<Option<Uuid>>,
    value: Signal<String>,
    saving: Signal<bool>,
) -> Element {
    let mut value = value;
    let mut saving = saving;

    let sub_name = sub_id()
        .map(|id| {
            SUBSCRIPTIONS()
                .subscriptions
                .iter()
                .find(|s| s.id == id)
                .map(|s| s.name.clone())
                .unwrap_or_default()
        })
        .unwrap_or_default();
    let title = format!("Notes: {}", sub_name);

    rsx! {
        Dialog {
            open: true,
            on_close: move |_| open.set(false),
            title: title,
            footer: rsx! {
                div { class: "flex flex-col-reverse sm:flex-row sm:justify-end gap-2",
                    button {
                        class: "border border-border/60 px-4 py-2 rounded-lg text-sm hover:bg-muted transition-colors duration-200",
                        onclick: move |_| open.set(false),
                        "Cancel"
                    }
                    button {
                        class: "bg-primary text-primary-foreground px-4 py-2 rounded-md text-sm hover:bg-primary/90",
                        disabled: saving(),
                        onclick: move |_| {
                            let id = sub_id().unwrap();
                            let notes = value();
                            saving.set(true);
                            spawn(async move {
                                let req = UpdateSubscriptionRequest {
                                    account_id: None,
                                    notes: Some(notes),
                                    ..Default::default()
                                };
                                let _ = Storage::update_subscription(&id, &req).await;
                                match Storage::list_subscriptions().await {
                                    Ok(data) => SUBSCRIPTIONS.write().set_data(data),
                                    Err(e) => SUBSCRIPTIONS.write().set_error(e.to_string()),
                                }
                                saving.set(false);
                                open.set(false);
                            });
                        },
                        if saving() { "Saving..." } else { "Save" }
                    }
                }
            },
            textarea {
                class: "w-full min-h-[120px] rounded-md border bg-background px-3 py-2 text-sm resize-none",
                placeholder: "Add notes...",
                value: value.to_string(),
                oninput: move |e: Event<FormData>| value.set(e.value()),
            }
        }
    }
}
