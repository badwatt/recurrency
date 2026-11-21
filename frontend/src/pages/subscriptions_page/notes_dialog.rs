use crate::components::button::{Button, ButtonVariant};
use crate::components::dialog::Dialog;
use crate::components::textarea::Textarea;
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
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| open.set(false),
                        "Cancel"
                    }
                    Button {
                        disabled: saving(),
                        onclick: move |_| {
                            let Some(id) = sub_id() else { return };
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
            Textarea {
                class: "min-h-[120px] resize-none",
                placeholder: "Add notes...",
                value: value.to_string(),
                on_input: move |next: String| value.set(next),
            }
        }
    }
}
