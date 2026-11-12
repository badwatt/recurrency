use crate::components::dialog::Dialog;
use crate::models::transaction::UpdateTransactionRequest;
use crate::state::transactions::TRANSACTIONS;
use crate::storage::Storage;
use dioxus::prelude::*;
use uuid::Uuid;

/// Read/edit a transaction's notes. Mounted fresh each time it opens, so the
/// textarea is initialised from the transaction referenced by `txn_id`.
#[component]
pub fn NotesDialog(open: Signal<bool>, txn_id: Signal<Option<Uuid>>) -> Element {
    let initial = txn_id()
        .and_then(|id| {
            TRANSACTIONS()
                .transactions
                .iter()
                .find(|t| t.id == id)
                .and_then(|t| t.notes.clone())
        })
        .unwrap_or_default();
    let mut notes_dialog_value = use_signal(move || initial);
    let mut notes_saving = use_signal(|| false);

    let txn_name = txn_id()
        .and_then(|id| {
            TRANSACTIONS()
                .transactions
                .iter()
                .find(|t| t.id == id)
                .map(|t| t.name.clone())
        })
        .unwrap_or_default();
    let title = format!("Notes: {}", txn_name);

    rsx! {
        Dialog {
            open: true,
            on_close: move |_| open.set(false),
            title: title,
            footer: rsx! {
                div { class: "flex flex-col-reverse sm:flex-row sm:justify-end gap-2",
                    button {
                        class: "border px-4 py-2 rounded-md text-sm hover:bg-muted",
                        onclick: move |_| open.set(false),
                        "Cancel"
                    }
                    button {
                        class: "bg-primary text-primary-foreground px-4 py-2 rounded-md text-sm hover:bg-primary/90",
                        disabled: notes_saving(),
                        onclick: move |_| {
                            let id = txn_id().unwrap();
                            let notes = notes_dialog_value();
                            notes_saving.set(true);
                            spawn(async move {
                                let req = UpdateTransactionRequest {
                                    account_id: None,
                                    notes: Some(notes),
                                    ..Default::default()
                                };
                                let _ = Storage::update_transaction(&id, &req).await;
                                match Storage::list_transactions().await {
                                    Ok(data) => TRANSACTIONS.write().set_data(data),
                                    Err(e) => TRANSACTIONS.write().set_error(e.to_string()),
                                }
                                notes_saving.set(false);
                                open.set(false);
                            });
                        },
                        if notes_saving() { "Saving..." } else { "Save" }
                    }
                }
            },
            textarea {
                class: "w-full min-h-[120px] rounded-md border bg-background px-3 py-2 text-sm resize-none",
                placeholder: "Add notes...",
                value: notes_dialog_value.to_string(),
                oninput: move |e: Event<FormData>| notes_dialog_value.set(e.value()),
            }
        }
    }
}
