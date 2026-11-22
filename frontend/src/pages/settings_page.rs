#![allow(clippy::suspicious_else_formatting)]
use crate::components::button::{Button, ButtonVariant};
use crate::components::card::SectionCard;
use crate::components::page_header::PageHeader;
use crate::components::segmented::{SegmentOption, SegmentVariant, SegmentedControl};
use crate::state::auth::AUTH;
use dioxus::prelude::*;

pub(crate) mod about_page;
mod ai_card;
mod ai_page;
mod appearance_card;
mod voice_card;
pub(crate) mod apk_card;
mod backup_card;
mod delete_account_card;
pub(crate) mod delivery_card;
pub(crate) mod exchange_rates_card;
pub(crate) mod preferences_card;
mod profile_card;
mod server_card;
pub(crate) mod summary_card;
pub(crate) mod sync_card;

use about_page::AboutPage;
use ai_card::AiConnectionCard;
use appearance_card::AppearanceCard;
use voice_card::AiVoiceCard;
use apk_card::ApkCard;
use backup_card::BackupCard;
use delete_account_card::DeleteAccountCard;
use delivery_card::DeliveryCard;
use exchange_rates_card::ExchangeRatesCard;
use preferences_card::PreferencesCard;
use profile_card::ProfileCard;
use server_card::ServerCard;
use summary_card::SummaryCard;
use sync_card::SyncFolderCard;

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
        div { class: "lg:hidden",
            SectionCard {
                title: "Session",
                description: "Sign out of this device",
                content_class: "flex items-center justify-between gap-4",
                p { class: "truncate text-sm", {email} }
                Button {
                    variant: ButtonVariant::Outline,
                    class: "shrink-0",
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
        div { class: "flex flex-col gap-6 page-enter",
            PageHeader { title: "Account", subtitle: "Your profile, session and account" }
            ProfileCard {}
            MobileSessionCard {}
            DeleteAccountCard {}
        }
    }
}

#[component]
pub fn SettingsPreferencesPage() -> Element {
    rsx! {
        div { class: "flex flex-col gap-6 page-enter",
            PageHeader { title: "General", subtitle: "Defaults, locale, exchange rates and app updates" }
            PreferencesCard {}
            ExchangeRatesCard {}
        }
    }
}

pub use ai_page::SettingsAiPage;

#[component]
pub fn SettingsAppearancePage() -> Element {
    rsx! {
        div { class: "flex flex-col gap-6 page-enter",
            PageHeader { title: "Appearance", subtitle: "Color theme and palette" }
            AppearanceCard {}
        }
    }
}

#[component]
pub fn SettingsMoneyPage() -> Element {
    let mut tab = use_signal(|| MoneyTab::Accounts);
    rsx! {
        div { class: "relative flex flex-col gap-6 page-enter",
            PageHeader { title: "Accounts & categories", subtitle: "Accounts and categories that organize your money" }

            SegmentedControl {
                value: (if tab() == MoneyTab::Accounts { "accounts" } else { "categories" }).to_string(),
                aria_label: "Accounts and categories sections".to_string(),
                class: "w-fit".to_string(),
                variant: SegmentVariant::Underline,
                options: vec![
                    SegmentOption::new("accounts", "Accounts"),
                    SegmentOption::new("categories", "Categories"),
                ],
                on_change: move |value| tab.set(if value == "accounts" { MoneyTab::Accounts } else { MoneyTab::Categories }),
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
pub fn SettingsDataPage() -> Element {
    rsx! {
        div { class: "flex flex-col gap-6 page-enter",
            PageHeader { title: "Data & backup", subtitle: "Keep a copy of your data and sync devices" }
            BackupCard {}
            SyncFolderCard {}
        }
    }
}

#[component]
pub fn SettingsServerPage() -> Element {
    rsx! {
        div { class: "flex flex-col gap-6 page-enter",
            PageHeader { title: "Server", subtitle: "Connection and device pairing" }
            ServerCard {}
        }
    }
}

#[component]
pub fn SettingsNotificationsPage() -> Element {
    let offline = crate::api::client::is_offline_mode();
    rsx! {
        div { class: "flex flex-col gap-6 page-enter",
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
    rsx! {
        div { class: "flex flex-col gap-6 page-enter",
            AboutPage {}
            ApkCard {}
        }
    }
}

#[component]
pub fn SettingsAdminPage() -> Element {
    rsx! { crate::pages::AdminPage {} }
}

#[cfg(test)]
mod tests;
