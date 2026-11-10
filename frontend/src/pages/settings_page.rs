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
                h2 { class: "text-lg font-semibold", "Accounts & categories" }
                p { class: "text-sm text-muted-foreground", "Open the lists that power your dashboard" }
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
        "Email reminders",
        "Receive email reminders before payments",
    ),
    (
        "push",
        "Push notifications",
        "Receive push notifications on your devices",
    ),
    (
        "system",
        "System notifications",
        "Receive desktop notifications",
    ),
];

pub fn is_email_disabled(channel: &str, smtp_configured: bool) -> bool {
    channel == "email" && !smtp_configured
}

/// App version shown at the bottom of Settings.
pub(crate) const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[allow(non_snake_case)]
fn SettingsVersion() -> Element {
    rsx! {
        p { class: "text-center text-xs text-muted-foreground/70 pt-1",
            {format!("Recurrency v{APP_VERSION}")}
        }
    }
}

#[component]
pub fn SettingsProfilePage() -> Element {
    rsx! {
        div { class: "p-2 md:p-6 animate-in fade-in duration-300",
            div { class: "w-full max-w-2xl mx-auto flex flex-col gap-6",
                PageHeader { title: "Profile", subtitle: "Update your personal information" }
                ProfileCard {}
                SettingsVersion {}
            }
        }
    }
}

#[component]
pub fn SettingsPreferencesPage() -> Element {
    rsx! {
        div { class: "p-2 md:p-6 animate-in fade-in duration-300",
            div { class: "w-full max-w-2xl mx-auto flex flex-col gap-6",
                PageHeader { title: "Preferences", subtitle: "Customize your experience" }
                PreferencesCard {}
                SettingsVersion {}
            }
        }
    }
}

#[component]
pub fn SettingsMoneyPage() -> Element {
    rsx! {
        div { class: "p-2 md:p-6 animate-in fade-in duration-300",
            div { class: "w-full max-w-3xl mx-auto flex flex-col gap-6",
                PageHeader { title: "Money", subtitle: "Accounts and categories that organize your money" }
                MoneyCard {}
                SettingsVersion {}
            }
        }
    }
}

#[component]
pub fn SettingsNotificationsPage() -> Element {
    let offline = crate::api::client::is_offline_mode();
    rsx! {
        div { class: "p-2 md:p-6 animate-in fade-in duration-300",
            div { class: "w-full max-w-5xl mx-auto flex flex-col gap-6",
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
                    div { class: "grid gap-6 lg:grid-cols-2 items-start",
                        SummaryCard {}
                        NotificationsCard {}
                    }
                }
                SettingsVersion {}
            }
        }
    }
}

#[component]
pub fn SettingsConnectionPage() -> Element {
    rsx! {
        div { class: "p-2 md:p-6 animate-in fade-in duration-300",
            div { class: "w-full max-w-2xl mx-auto flex flex-col gap-6",
                PageHeader { title: "Connection", subtitle: "Manage how Recurrency reaches your data" }
                ConnectionCard {}
                SettingsVersion {}
            }
        }
    }
}

#[cfg(test)]
mod tests;
