#![allow(clippy::suspicious_else_formatting)]
use crate::components::page_header::PageHeader;
use crate::state::auth::AUTH;
use dioxus::prelude::*;

mod about_page;
mod apk_card;
mod backup_card;
mod sync_card;
mod delete_account_card;
mod delivery_card;
mod preferences_card;
mod profile_card;
mod summary_card;

use about_page::AboutPage;
use apk_card::ApkCard;
use backup_card::BackupCard;
use sync_card::SyncFolderCard;
use delete_account_card::DeleteAccountCard;
use delivery_card::DeliveryCard;
use preferences_card::PreferencesCard;
use profile_card::ProfileCard;
use summary_card::SummaryCard;

#[derive(Clone, Copy, PartialEq, Eq)]
enum MoneyTab {
    Accounts,
    Categories,
}

/// App version shown in the settings sidebar.
pub(crate) const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[component]
fn MobileSessionCard() -> Element {
    let nav = navigator();
    let email = AUTH()
        .user
        .as_ref()
        .map(|user| user.email.clone())
        .unwrap_or_default();

    rsx! {
        div { class: "lg:hidden rounded-xl bg-card/80 border border-border/40",
            div { class: "p-6 border-b border-border",
                h2 { class: "text-lg font-semibold", "Session" }
                p { class: "text-sm text-muted-foreground", "Sign out of this device" }
            }
            div { class: "p-6 flex items-center justify-between gap-4",
                p { class: "text-sm truncate", {email} }
                button {
                    r#type: "button",
                    class: "inline-flex h-9 shrink-0 items-center justify-center rounded-md border border-border px-3 text-sm font-medium hover:bg-accent",
                    onclick: move |_| {
                        AUTH.write().logout();
                        AUTH.write().persist_to_storage();
                        nav.push(crate::Route::Login {});
                    },
                    "Log out"
                }
            }
        }
    }
}

#[component]
pub fn SettingsProfilePage() -> Element {
    rsx! {
        div { class: "flex flex-col gap-6 animate-in fade-in duration-300",
            PageHeader { title: "Profile", subtitle: "Your identity and session" }
            ProfileCard {}
            ApkCard {}
            BackupCard {}
            SyncFolderCard {}
            MobileSessionCard {}
            DeleteAccountCard {}
        }
    }
}

#[component]
pub fn SettingsPreferencesPage() -> Element {
    rsx! {
        div { class: "flex flex-col gap-6 animate-in fade-in duration-300",
            PageHeader { title: "Preferences", subtitle: "Defaults applied across Recurrency" }
            PreferencesCard {}
        }
    }
}

#[component]
pub fn SettingsMoneyPage() -> Element {
    let mut tab = use_signal(|| MoneyTab::Accounts);
    rsx! {
        div { class: "flex flex-col gap-6 animate-in fade-in duration-300",
            PageHeader { title: "Money", subtitle: "Accounts and categories that organize your money" }

            div { class: "flex items-center gap-1 border-b border-border/50", role: "tablist", aria_label: "Money sections",
                button {
                    r#type: "button",
                    role: "tab",
                    aria_selected: if tab() == MoneyTab::Accounts { "true" } else { "false" },
                    class: if tab() == MoneyTab::Accounts {
                        "inline-flex items-center gap-2 border-b-2 border-primary px-3 py-2 text-sm font-semibold text-foreground"
                    } else {
                        "inline-flex items-center gap-2 border-b-2 border-transparent px-3 py-2 text-sm font-medium text-muted-foreground hover:text-foreground"
                    },
                    onclick: move |_| tab.set(MoneyTab::Accounts),
                    "Accounts"
                }
                button {
                    r#type: "button",
                    role: "tab",
                    aria_selected: if tab() == MoneyTab::Categories { "true" } else { "false" },
                    class: if tab() == MoneyTab::Categories {
                        "inline-flex items-center gap-2 border-b-2 border-primary px-3 py-2 text-sm font-semibold text-foreground"
                    } else {
                        "inline-flex items-center gap-2 border-b-2 border-transparent px-3 py-2 text-sm font-medium text-muted-foreground hover:text-foreground"
                    },
                    onclick: move |_| tab.set(MoneyTab::Categories),
                    "Categories"
                }
            }

            if tab() == MoneyTab::Accounts {
                crate::pages::AccountsPage { embedded: true }
            } else {
                crate::pages::CategoriesPage { embedded: true }
            }
        }
    }
}

#[component]
pub fn SettingsNotificationsPage() -> Element {
    let offline = crate::api::client::is_offline_mode();
    rsx! {
        div { class: "flex flex-col gap-6 animate-in fade-in duration-300",
            PageHeader { title: "Notifications", subtitle: "Choose where your summary lands and when it leaves" }
            if offline {
                div { class: "rounded-xl bg-card/80 border border-border/40 p-6",
                    h2 { class: "text-lg font-semibold", "Server notifications" }
                    p { class: "text-sm text-muted-foreground mt-1",
                        "Delivery channels and summary schedules require server mode. Set a server URL in Preferences."
                    }
                }
            } else {
                DeliveryCard {}
                SummaryCard {}
            }
        }
    }
}


#[component]
pub fn SettingsAboutPage() -> Element {
    rsx! { AboutPage {} }
}

#[cfg(test)]
mod tests;
