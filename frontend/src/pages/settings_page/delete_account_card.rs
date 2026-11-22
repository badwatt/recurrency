use crate::api::settings;
use crate::components::dialog::Dialog;
use crate::models::auth::DeleteAccountRequest;
use crate::state::auth::AUTH;
use dioxus::prelude::*;

use crate::components::button::{Button, ButtonVariant};
use crate::components::card::SectionCard;
use crate::components::field::Field;
use crate::components::input::Input;
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
        SectionCard {
            title: "Delete account",
            description: "Permanently erase your account and all of its data",
            class: "border-destructive/30",
            title_class: "text-destructive",
            header_class: "border-destructive/20",
            content_class: "p-0",
            div { class: "p-6 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between",
                div { class: "min-w-0",
                    p { class: "text-sm font-medium truncate", {email} }
                    p { class: "text-xs text-muted-foreground", "This cannot be undone." }
                }
                Button {
                    variant: ButtonVariant::Outline,
                    class: "border-destructive/40 text-destructive hover:bg-destructive/10",
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
                Field { label: "Confirm with your password", html_for: "delete-account-password",
                    Input {
                        id: "delete-account-password",
                        r#type: "password",
                        autocomplete: "current-password",
                        value: password.to_string(),
                        on_input: move |value| password.set(value),
                    }
                }
                if !error().is_empty() {
                    p { class: "text-sm text-destructive", role: "alert", {error()} }
                }
                div { class: "flex flex-col-reverse gap-2 pt-2 sm:flex-row sm:justify-end",
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| {
                            open.set(false);
                            password.set(String::new());
                            error.set(String::new());
                        },
                        "Cancel"
                    }
                    Button {
                        variant: ButtonVariant::Destructive,
                        disabled: busy(),
                        onclick: delete,
                        if busy() { "Deleting..." } else { "Delete account" }
                    }
                }
            }
        }
    }
}
