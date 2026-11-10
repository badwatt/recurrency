use crate::api::settings;
use crate::components::dialog::Dialog;
use crate::models::auth::DeleteAccountRequest;
use crate::state::auth::AUTH;
use dioxus::prelude::*;

#[component]
pub fn DeleteAccountCard() -> Element {
    let nav = navigator();
    let mut open = use_signal(|| false);
    let mut password = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(String::new);
    let email = AUTH()
        .user
        .as_ref()
        .map(|u| u.email.clone())
        .unwrap_or_default();

    let delete = move |_| {
        let pwd = password();
        if pwd.is_empty() {
            error.set("Enter your password to confirm".to_string());
            return;
        }
        error.set(String::new());
        busy.set(true);
        spawn(async move {
            match settings::delete_account(&DeleteAccountRequest { password: pwd }).await {
                Ok(()) => {
                    AUTH.write().logout();
                    AUTH.write().persist_to_storage();
                    nav.push(crate::Route::Login {});
                }
                Err(e) => {
                    error.set(e.to_string());
                    busy.set(false);
                }
            }
        });
    };

    rsx! {
        div { class: "rounded-xl border border-destructive/30 bg-card/80",
            div { class: "p-6 border-b border-destructive/20",
                h2 { class: "text-lg font-semibold text-destructive", "Delete account" }
                p { class: "text-sm text-muted-foreground", "Permanently erase your account and all of its data" }
            }
            div { class: "p-6 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between",
                div { class: "min-w-0",
                    p { class: "text-sm font-medium truncate", {email} }
                    p { class: "text-xs text-muted-foreground", "This cannot be undone." }
                }
                button {
                    r#type: "button",
                    class: "inline-flex h-10 items-center justify-center rounded-md border border-destructive/40 px-4 text-sm font-medium text-destructive hover:bg-destructive/10 transition-colors",
                    onclick: move |_| open.set(true),
                    "Delete account"
                }
            }
        }

        Dialog {
            open: open(),
            on_close: move |_| {
                open.set(false);
                password.set(String::new());
                error.set(String::new());
            },
            title: "Delete account".to_string(),
            div { class: "space-y-4",
                p { class: "text-sm text-muted-foreground",
                    "This permanently deletes your account, subscriptions, transactions, accounts, categories and notification settings."
                }
                div { class: "space-y-2",
                    label { r#for: "delete-account-password", class: "text-sm font-medium", "Confirm with your password" }
                    input {
                        id: "delete-account-password",
                        r#type: "password",
                        autocomplete: "current-password",
                        value: password.to_string(),
                        oninput: move |e| password.set(e.value()),
                        class: "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm",
                    }
                }
                if !error().is_empty() {
                    p { class: "text-sm text-destructive", role: "alert", {error()} }
                }
                div { class: "flex flex-col-reverse gap-2 pt-2 sm:flex-row sm:justify-end",
                    button {
                        r#type: "button",
                        class: "h-10 rounded-md border border-border px-4 text-sm font-medium hover:bg-accent",
                        onclick: move |_| {
                            open.set(false);
                            password.set(String::new());
                            error.set(String::new());
                        },
                        "Cancel"
                    }
                    button {
                        r#type: "button",
                        class: "h-10 rounded-md bg-destructive px-4 text-sm font-medium text-white hover:bg-destructive/90 disabled:opacity-50",
                        disabled: busy(),
                        onclick: delete,
                        if busy() { "Deleting..." } else { "Delete account" }
                    }
                }
            }
        }
    }
}
