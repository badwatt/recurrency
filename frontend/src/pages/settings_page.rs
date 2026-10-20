#![allow(clippy::suspicious_else_formatting)]
use crate::components::page_header::PageHeader;
use dioxus::prelude::*;

mod connection_card;
mod notifications_card;
mod preferences_card;
mod profile_card;

use connection_card::ConnectionCard;
use notifications_card::NotificationsCard;
use preferences_card::PreferencesCard;
use profile_card::ProfileCard;

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

#[component]
pub fn SettingsPage() -> Element {
    rsx! {
        div { class: "p-2 md:p-6 flex flex-col gap-6 animate-in fade-in duration-300",
            div { class: "",
                PageHeader { title: "Settings", subtitle: "Manage your account and preferences" }
            }

            ProfileCard {}
            PreferencesCard {}
            ConnectionCard {}
            NotificationsCard {}
        }
    }
}

#[cfg(test)]
mod tests;
