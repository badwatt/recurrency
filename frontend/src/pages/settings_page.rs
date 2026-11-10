#![allow(clippy::suspicious_else_formatting)]
use crate::components::page_header::PageHeader;
use dioxus::prelude::*;

mod connection_card;
mod delivery_card;
mod notifications_card;
mod preferences_card;
mod profile_card;
mod summary_card;

use connection_card::ConnectionCard;
use delivery_card::DeliveryCard;
use notifications_card::NotificationsCard;
use preferences_card::PreferencesCard;
use profile_card::ProfileCard;
use summary_card::SummaryCard;

#[component]
fn MoneyCard() -> Element {
    rsx! {
        div { class: "rounded-xl bg-card/80 border border-border/40",
            div { class: "p-6 border-b border-border",
                h2 { class: "text-lg font-semibold", "Money" }
                p { class: "text-sm text-muted-foreground", "Manage the accounts and categories that organize your money" }
            }
            div { class: "p-6 grid gap-3 sm:grid-cols-2",
                Link {
                    to: crate::Route::SettingsAccounts {},
                    class: "rounded-lg border border-border/50 bg-background/40 p-4 hover:bg-accent transition-colors",
                    p { class: "font-medium", "Accounts" }
                    p { class: "text-sm text-muted-foreground mt-1", "Where you move money each month" }
                }
                Link {
                    to: crate::Route::SettingsCategories {},
                    class: "rounded-lg border border-border/50 bg-background/40 p-4 hover:bg-accent transition-colors",
                    p { class: "font-medium", "Categories" }
                    p { class: "text-sm text-muted-foreground mt-1", "Optional labels for your spending" }
                }
            }
        }
    }
}

const NOTIFICATION_CHANNELS: &[(&str, &str, &str)] = &[
    (
        "email",
        "Email Notifications",
        "Receive email reminders before payments",
    ),
    (
        "push",
        "Push Notifications",
        "Receive push notifications on your devices",
    ),
    (
        "system",
        "System Notifications",
        "Receive desktop notifications",
    ),
];

pub fn is_email_disabled(channel: &str, smtp_configured: bool) -> bool {
    channel == "email" && !smtp_configured
}

/// App version shown at the bottom of Settings.
pub(crate) const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[component]
pub fn SettingsPage() -> Element {
    rsx! {
        div { class: "p-2 md:p-6 flex flex-col gap-6 animate-in fade-in duration-300",
            div { class: "",
                PageHeader { title: "Settings", subtitle: "Manage your account and preferences" }
            }

            ProfileCard {}
            PreferencesCard {}
            MoneyCard {}
            ConnectionCard {}
            NotificationsCard {}
            DeliveryCard {}
            SummaryCard {}

            p { class: "text-center text-xs text-muted-foreground/70 pt-1",
                {format!("Recurrency v{APP_VERSION}")}
            }
        }
    }
}

#[cfg(test)]
mod tests;
