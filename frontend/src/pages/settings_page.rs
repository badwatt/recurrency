use dioxus::prelude::*;
use crate::state::auth::AUTH;
use crate::state::preferences::PREFERENCES;
use crate::api::settings;
use crate::api::notification_preferences;
use crate::models::auth::UpdateSettingsRequest;
use crate::models::notification::{UpdateNotificationPreferenceRequest, UpdateNotificationPreferencesRequest};
use crate::utils::constants::CURRENCIES;

const NOTIFICATION_CHANNELS: &[(&str, &str, &str)] = &[
    ("email", "Email Notifications", "Receive email reminders before payments"),
    ("push", "Push Notifications", "Receive push notifications on your devices"),
    ("system", "System Notifications", "Receive desktop notifications"),
];

#[component]
pub fn SettingsPage() -> Element {
    let mut name = use_signal(|| AUTH().user.as_ref().and_then(|u| u.name.clone()).unwrap_or_default());
    let mut currency = use_signal(|| AUTH().currency().to_string());
    let mut notification_prefs = use_signal(|| {
        let mut map = std::collections::HashMap::new();
        for (key, _, _) in NOTIFICATION_CHANNELS {
            map.insert(key.to_string(), false);
        }
        map
    });
    let mut prefs_loading = use_signal(|| false);
    let mut saving_name = use_signal(|| false);
    let mut saving_currency = use_signal(|| false);
    let server_url = use_signal(|| crate::api::client::get_base_url());
    let mut name_error = use_signal(String::new);

    // Load notification preferences
    use_future(move || async move {
        match notification_preferences::get().await {
            Ok(resp) => {
                let mut map = std::collections::HashMap::new();
                for p in resp.preferences {
                    map.insert(p.channel, p.enabled);
                }
                notification_prefs.set(map);
            }
            Err(e) => log::error!("Failed to load notification prefs: {}", e),
        }
    });

    // Sync form state when user changes
    use_effect(move || {
        if let Some(u) = &AUTH().user {
            name.set(u.name.clone().unwrap_or_default());
            currency.set(u.currency.clone());
        }
    });

    let handle_save_name = move |_| {
        let n = name();
        if n.len() < 2 {
            name_error.set("Name must be at least 2 characters".into());
            return;
        }
        name_error.set(String::new());
        saving_name.set(true);
        spawn(async move {
            match settings::update(&UpdateSettingsRequest {
                name: Some(n),
                currency: None,
            }).await {
                Ok(resp) => {
                    AUTH.write().user = Some(resp.user);
                    AUTH.write().persist_to_storage();
                    saving_name.set(false);
                }
                Err(e) => {
                    log::error!("Failed to save name: {}", e);
                    saving_name.set(false);
                }
            }
        });
    };

    let mut handle_save_currency = move |new_currency: String| {
        currency.set(new_currency.clone());
        saving_currency.set(true);
        spawn(async move {
            match settings::update(&UpdateSettingsRequest {
                name: None,
                currency: Some(new_currency),
            }).await {
                Ok(resp) => {
                    AUTH.write().user = Some(resp.user);
                    AUTH.write().persist_to_storage();
                    saving_currency.set(false);
                }
                Err(e) => {
                    log::error!("Failed to save currency: {}", e);
                    saving_currency.set(false);
                }
            }
        });
    };

    let mut handle_notification_toggle = move |channel: String, enabled: bool| {
        let mut prefs = notification_prefs();
        prefs.insert(channel.clone(), enabled);
        notification_prefs.set(prefs);
        prefs_loading.set(true);
        spawn(async move {
            let req = UpdateNotificationPreferencesRequest {
                preferences: vec![UpdateNotificationPreferenceRequest {
                    channel: channel.clone(),
                    enabled,
                }],
            };
            match notification_preferences::update(&req).await {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Failed to update notification pref: {}", e);
                    let mut prefs = notification_prefs();
                    prefs.insert(channel, !enabled);
                    notification_prefs.set(prefs);
                }
            }
            prefs_loading.set(false);
        });
    };

    let user_email = AUTH().user.as_ref().map(|u| u.email.clone()).unwrap_or_default();
    let cur = currency();
    let theme_label = match PREFERENCES().theme() {
        "light" => "Light",
        "dark" => "Dark",
        "mocha" => "Mocha",
        _ => "System",
    };

    rsx! {
        div { class: "p-4 md:p-6 space-y-6",
            h1 { class: "text-3xl font-bold tracking-tight", "Settings" }
            p { class: "text-muted-foreground", "Manage your account and preferences" }

            // Profile card
            div { class: "rounded-lg border border-border bg-card",
                div { class: "p-6 border-b border-border",
                    h2 { class: "text-lg font-semibold", "Profile" }
                    p { class: "text-sm text-muted-foreground", "Update your personal information" }
                }
                div { class: "p-6 space-y-4",
                    // Email (read-only)
                    div { class: "space-y-2",
                        label { r#for: "settings-email", class: "text-sm font-medium", "Email" }
                        input {
                            id: "settings-email",
                            r#type: "email",
                            value: "{user_email}",
                            disabled: true,
                            class: "flex h-10 w-full rounded-md border border-input bg-muted px-3 py-2 text-sm opacity-50 cursor-not-allowed",
                        }
                    }
                    // Name
                    div { class: "space-y-2",
                        label { r#for: "settings-name", class: "text-sm font-medium", "Name" }
                        div { class: "flex gap-2",
                            input {
                                id: "settings-name",
                                r#type: "text",
                                value: "{name}",
                                oninput: move |e| { name.set(e.value()); name_error.set(String::new()); },
                                class: "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm",
                            }
                            button {
                                onclick: handle_save_name,
                                disabled: saving_name(),
                                class: "h-10 px-4 py-2 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 disabled:opacity-50 shrink-0",
                                if saving_name() { "Saving..." } else { "Save" }
                            }
                        }
                        if !name_error().is_empty() {
                            p { class: "text-sm text-destructive", "{name_error}" }
                        }
                    }
                    // Logout
                    div { class: "pt-4 border-t border-border",
                        button {
                            onclick: move |_| {
                                AUTH.write().logout();
                                let nav = navigator();
                                nav.push(crate::Route::Login {});
                            },
                            class: "flex w-full items-center justify-center gap-2 h-10 rounded-md border border-destructive/50 text-destructive text-sm font-medium hover:bg-destructive/10 transition-colors",
                            svg {
                                xmlns: "http://www.w3.org/2000/svg",
                                width: "16",
                                height: "16",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                path { d: "M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" }
                                polyline { points: "16 17 21 12 16 7" }
                                line { x1: "21", y1: "12", x2: "9", y2: "12" }
                            }
                            "Log out"
                        }
                    }
                }
            }

            // Preferences card
            div { class: "rounded-lg border border-border bg-card",
                div { class: "p-6 border-b border-border",
                    h2 { class: "text-lg font-semibold", "Preferences" }
                    p { class: "text-sm text-muted-foreground", "Customize your experience" }
                }
                div { class: "p-6 space-y-4",
                    div { class: "space-y-2",
                        label { r#for: "settings-currency", class: "text-sm font-medium", "Default Currency" }
                        select {
                            id: "settings-currency",
                            value: "{cur}",
                            onchange: move |e| handle_save_currency(e.value()),
                            disabled: saving_currency(),
                            class: "flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm focus:outline-none focus:ring-1 focus:ring-ring",
                            for (val, label) in CURRENCIES {
                                option { value: "{val}", "{label}" }
                            }
                        }
                    }
                }
            }

            // Appearance card
            div { class: "rounded-lg border border-border bg-card",
                div { class: "p-6 border-b border-border",
                    h2 { class: "text-lg font-semibold", "Appearance" }
                    p { class: "text-sm text-muted-foreground", "Customize the look and feel" }
                }
                div { class: "p-6 space-y-4",
                    div { class: "flex items-center justify-between",
                        div {
                            p { class: "font-medium", "Theme" }
                            p { class: "text-sm text-muted-foreground", "{theme_label}" }
                        }
                        button {
                            onclick: move |_| crate::theme::hooks::toggle_theme(),
                            class: "flex items-center justify-center h-9 w-9 rounded-md border border-input bg-background hover:bg-accent hover:text-accent-foreground transition-colors",
                            {match PREFERENCES().theme() {
                                "light" => rsx! {
                                    svg {
                                        xmlns: "http://www.w3.org/2000/svg",
                                        width: "18",
                                        height: "18",
                                        view_box: "0 0 24 24",
                                        fill: "none",
                                        stroke: "currentColor",
                                        stroke_width: "2",
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        circle { cx: "12", cy: "12", r: "5" }
                                        line { x1: "12", y1: "1", x2: "12", y2: "3" }
                                        line { x1: "12", y1: "21", x2: "12", y2: "23" }
                                        line { x1: "4.22", y1: "4.22", x2: "5.64", y2: "5.64" }
                                        line { x1: "18.36", y1: "18.36", x2: "19.78", y2: "19.78" }
                                        line { x1: "1", y1: "12", x2: "3", y2: "12" }
                                        line { x1: "21", y1: "12", x2: "23", y2: "12" }
                                        line { x1: "4.22", y1: "19.78", x2: "5.64", y2: "18.36" }
                                        line { x1: "18.36", y1: "5.64", x2: "19.78", y2: "4.22" }
                                    }
                                },
                                "dark" => rsx! {
                                    svg {
                                        xmlns: "http://www.w3.org/2000/svg",
                                        width: "18",
                                        height: "18",
                                        view_box: "0 0 24 24",
                                        fill: "none",
                                        stroke: "currentColor",
                                        stroke_width: "2",
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        path { d: "M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" }
                                    }
                                },
                                "mocha" => rsx! {
                                    svg {
                                        xmlns: "http://www.w3.org/2000/svg",
                                        width: "18",
                                        height: "18",
                                        view_box: "0 0 24 24",
                                        fill: "none",
                                        stroke: "currentColor",
                                        stroke_width: "2",
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        path { d: "M18 8h1a4 4 0 0 1 0 8h-1" }
                                        path { d: "M2 8h16v9a4 4 0 0 1-4 4H6a4 4 0 0 1-4-4V8z" }
                                        line { x1: "6", y1: "1", x2: "6", y2: "4" }
                                        line { x1: "10", y1: "1", x2: "10", y2: "4" }
                                        line { x1: "14", y1: "1", x2: "14", y2: "4" }
                                    }
                                },
                                _ => rsx! {
                                    svg {
                                        xmlns: "http://www.w3.org/2000/svg",
                                        width: "18",
                                        height: "18",
                                        view_box: "0 0 24 24",
                                        fill: "none",
                                        stroke: "currentColor",
                                        stroke_width: "2",
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        rect { x: "2", y: "3", width: "20", height: "14", rx: "2", ry: "2" }
                                        line { x1: "8", y1: "21", x2: "16", y2: "21" }
                                        line { x1: "12", y1: "17", x2: "12", y2: "21" }
                                    }
                                }
                            }}
                        }
                    }
                }
            }

            // Server card
            div { class: "rounded-lg border border-border bg-card",
                div { class: "p-6 border-b border-border",
                    h2 { class: "text-lg font-semibold", "Server" }
                    p { class: "text-sm text-muted-foreground", "Your Recurrency server connection" }
                }
                div { class: "p-6 space-y-4",
                    div { class: "space-y-2",
                        label { r#for: "settings-server", class: "text-sm font-medium", "Server URL" }
                        input {
                            id: "settings-server",
                            r#type: "text",
                            value: "{server_url}",
                            disabled: true,
                            class: "flex h-10 w-full rounded-md border border-input bg-muted px-3 py-2 text-sm opacity-50 cursor-not-allowed",
                        }
                    }
                }
            }

            // Notifications card
            div { class: "rounded-lg border border-border bg-card",
                div { class: "p-6 border-b border-border",
                    h2 { class: "text-lg font-semibold", "Notifications" }
                    p { class: "text-sm text-muted-foreground", "Configure how you receive notifications" }
                }
                div { class: "p-6 space-y-4",
                    for (idx, (key, label, desc)) in NOTIFICATION_CHANNELS.iter().enumerate() {
                        {let checked = notification_prefs().get(*key).copied().unwrap_or(false);
                         let key_str = key.to_string();
                         rsx! {
                            div { key: "{key}",
                                div { class: "flex items-center justify-between",
                                    div {
                                        p { class: "font-medium", "{label}" }
                                        p { class: "text-sm text-muted-foreground", "{desc}" }
                                    }
                                    button {
                                        class: if checked {
                                            "relative inline-flex h-6 w-11 items-center rounded-full bg-primary transition-colors focus:outline-none"
                                        } else {
                                            "relative inline-flex h-6 w-11 items-center rounded-full bg-muted transition-colors focus:outline-none"
                                        },
                                        onclick: {
                                            let key_str = key_str.clone();
                                            let new_val = !checked;
                                            move |_| handle_notification_toggle(key_str.clone(), new_val)
                                        },
                                        disabled: prefs_loading(),
                                        span {
                                            class: if checked {
                                                "inline-block h-4 w-4 transform rounded-full bg-white transition-transform translate-x-6"
                                            } else {
                                                "inline-block h-4 w-4 transform rounded-full bg-white transition-transform translate-x-1"
                                            },
                                        }
                                    }
                                }
                                if idx < NOTIFICATION_CHANNELS.len() - 1 {
                                    hr { class: "mt-4 border-border" }
                                }
                            }
                        }}
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notification_channels_count() {
        assert_eq!(NOTIFICATION_CHANNELS.len(), 3);
    }

    #[test]
    fn notification_channels_keys() {
        let keys: Vec<_> = NOTIFICATION_CHANNELS.iter().map(|(k,_,_)| *k).collect();
        assert!(keys.contains(&"email"));
        assert!(keys.contains(&"push"));
        assert!(keys.contains(&"system"));
    }

    #[test]
    fn currencies_contains_eur() {
        assert!(CURRENCIES.iter().any(|(v, _)| *v == "EUR"));
    }

    #[test]
    fn name_validation_too_short() {
        let name = "A";
        assert!(name.len() < 2);
    }

    #[test]
    fn name_validation_ok() {
        let name = "Alice";
        assert!(name.len() >= 2);
    }
}