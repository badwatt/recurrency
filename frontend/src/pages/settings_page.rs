#![allow(clippy::suspicious_else_formatting)]
use crate::components::page_header::PageHeader;
use dioxus::prelude::*;

mod connection_card;
mod delete_account_card;
mod delivery_card;
mod preferences_card;
mod profile_card;
mod summary_card;

use connection_card::ConnectionCard;
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
pub fn SettingsProfilePage() -> Element {
    rsx! {
        div { class: "flex flex-col gap-6 animate-in fade-in duration-300",
            PageHeader { title: "Profile", subtitle: "Your identity and session" }
            ProfileCard {}
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
                        "Delivery channels and summary schedules require server mode. Switch connection mode to configure them."
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
pub fn SettingsConnectionPage() -> Element {
    rsx! {
        div { class: "flex flex-col gap-6 animate-in fade-in duration-300",
            PageHeader { title: "Connection", subtitle: "Manage how Recurrency reaches your data" }
            ConnectionCard {}
        }
    }
}

#[cfg(test)]
mod tests;
